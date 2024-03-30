use crate::cli::Context;
use std::fs::read_to_string;
use thiserror::Error;

mod ppp; // precise point positioning
use ppp::post_process as ppp_post_process;
use ppp::PostProcessingError as PPPPostProcessingError;

//mod cggtts; // CGGTTS special solver
//use cggtts::post_process as cggtts_post_process;
//use cggtts::PostProcessingError as CGGTTSPostProcessingError;

use clap::ArgMatches;
use gnss::prelude::{Constellation, SV};

use rinex::{
    carrier::Carrier,
    navigation::Ephemeris,
    observation::ObservationData,
    prelude::{EpochFlag, Observable, Rinex},
};

use std::collections::{BTreeMap, HashMap};

use rtk::prelude::{
    AprioriPosition, BdModel, Clock, ClockIter as RTKClockIter, Config, Duration, Epoch, KbModel,
    Method, NgModel, Observation, ObservationIter as RTKObservationIter, PVTSolutionType, Solver,
    Vector3,
};

use map_3d::{ecef2geodetic, rad2deg, Ellipsoid};

/// Efficient Observation stream
struct ObservationIter<'a> {
    iter: Box<dyn Iterator<Item = (Epoch, SV, f64, f64)> + 'a>,
}

impl<'a> ObservationIter<'a> {
    fn from_ctx(
        pseudo_range: Box<dyn Iterator<Item = (Epoch, EpochFlag, SV, &'a Observable, f64)> + 'a>,
    ) -> Self {
        Self {
            iter: Box::new(pseudo_range.filter_map(|(e, flag, sv, observable, data)| {
                if flag.is_ok() {
                    if let Ok(carrier) = Carrier::from_observable(sv.constellation, observable) {
                        Some((e, sv, carrier.frequency(), data))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })),
        }
    }
}

impl<'a> RTKObservationIter for ObservationIter<'a> {
    fn next(&mut self) -> Option<Observation> {
        self.iter
            .next()
            .map(|(t, sv, freq_hz, data)| Observation::new(sv, t, data, freq_hz, None))
    }
}

/// Efficient Clock state stream
struct ClockIter<'a> {
    iter: Box<dyn Iterator<Item = (Epoch, SV, f64, Option<f64>, Option<f64>)> + 'a>,
}

impl<'a> ClockIter<'a> {
    fn from_ctx(ctx: &'a Context) -> Self {
        Self {
            iter: if let Some(clk) = ctx.data.clock() {
                Box::new(clk.precise_sv_clock().map(|(t, sv, _, profile)| {
                    (t, sv, profile.bias, profile.drift, profile.drift_change)
                }))
            } else {
                panic!("not yet")
            },
        }
    }
}

impl<'a> RTKClockIter for ClockIter<'a> {
    fn next(&mut self) -> Option<Clock> {
        self.iter
            .next()
            .map(|(sv, t, offset, drift, drift_r)| Clock::new(t, sv, offset, drift, drift_r))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("solver error")]
    SolverError(#[from] rtk::Error),
    #[error("undefined apriori position")]
    UndefinedAprioriPosition,
    #[error("ppp post processing error")]
    PPPPostProcessingError(#[from] PPPPostProcessingError),
    // #[error("cggtts post processing error")]
    // CGGTTSPostProcessingError(#[from] CGGTTSPostProcessingError),
}

pub fn tropo_components(meteo: Option<&Rinex>, t: Epoch, lat_ddeg: f64) -> Option<(f64, f64)> {
    const MAX_LATDDEG_DELTA: f64 = 15.0;
    let max_dt = Duration::from_hours(24.0);
    let rnx = meteo?;
    let meteo = rnx.header.meteo.as_ref().unwrap();

    let delays: Vec<(Observable, f64)> = meteo
        .sensors
        .iter()
        .filter_map(|s| match s.observable {
            Observable::ZenithDryDelay => {
                let (x, y, z, _) = s.position?;
                let (lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                let lat = rad2deg(lat);
                if (lat - lat_ddeg).abs() < MAX_LATDDEG_DELTA {
                    let value = rnx
                        .zenith_dry_delay()
                        .filter(|(t_sens, _)| (*t_sens - t).abs() < max_dt)
                        .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
                    let (_, value) = value?;
                    debug!("{:?} lat={} zdd {}", t, lat_ddeg, value);
                    Some((s.observable.clone(), value))
                } else {
                    None
                }
            },
            Observable::ZenithWetDelay => {
                let (x, y, z, _) = s.position?;
                let (mut lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                lat = rad2deg(lat);
                if (lat - lat_ddeg).abs() < MAX_LATDDEG_DELTA {
                    let value = rnx
                        .zenith_wet_delay()
                        .filter(|(t_sens, _)| (*t_sens - t).abs() < max_dt)
                        .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
                    let (_, value) = value?;
                    debug!("{:?} lat={} zdd {}", t, lat_ddeg, value);
                    Some((s.observable.clone(), value))
                } else {
                    None
                }
            },
            _ => None,
        })
        .collect();

    if delays.len() < 2 {
        None
    } else {
        let zdd = delays
            .iter()
            .filter_map(|(obs, value)| {
                if obs == &Observable::ZenithDryDelay {
                    Some(*value)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
            .unwrap();

        let zwd = delays
            .iter()
            .filter_map(|(obs, value)| {
                if obs == &Observable::ZenithWetDelay {
                    Some(*value)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
            .unwrap();

        Some((zwd, zdd))
    }
}

/*
 * Grabs nearest KB model (in time)
 */
pub fn kb_model(nav: &Rinex, t: Epoch) -> Option<KbModel> {
    let kb_model = nav
        .klobuchar_models()
        .min_by_key(|(t_i, _, _)| (t - *t_i).abs());

    if let Some((_, sv, kb_model)) = kb_model {
        Some(KbModel {
            h_km: {
                match sv.constellation {
                    Constellation::BeiDou => 375.0,
                    // we only expect GPS or BDS here,
                    // badly formed RINEX will generate errors in the solutions
                    _ => 350.0,
                }
            },
            alpha: kb_model.alpha,
            beta: kb_model.beta,
        })
    } else {
        /* RINEX 3 case */
        let iono_corr = nav.header.ionod_correction?;
        iono_corr.as_klobuchar().map(|kb_model| KbModel {
            h_km: 350.0, //TODO improve this
            alpha: kb_model.alpha,
            beta: kb_model.beta,
        })
    }
}

/*
 * Grabs nearest BD model (in time)
 */
pub fn bd_model(nav: &Rinex, t: Epoch) -> Option<BdModel> {
    nav.bdgim_models()
        .min_by_key(|(t_i, _)| (t - *t_i).abs())
        .map(|(_, model)| BdModel { alpha: model.alpha })
}

/*
 * Grabs nearest NG model (in time)
 */
pub fn ng_model(nav: &Rinex, t: Epoch) -> Option<NgModel> {
    nav.nequick_g_models()
        .min_by_key(|(t_i, _)| (t - *t_i).abs())
        .map(|(_, model)| NgModel { a: model.a })
}

pub fn precise_positioning(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    /* Resolution method */
    let method = match matches.get_flag("spp") {
        true => Method::SPP,
        false => Method::PPP,
    };

    let cfg = match matches.get_one::<String>("cfg") {
        Some(fp) => {
            let content = read_to_string(fp)
                .unwrap_or_else(|_| panic!("failed to read configuration: permission denied"));
            let cfg = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("failed to parse configuration: invalid content"));
            info!("Using custom solver configuration: {:#?}", cfg);
            cfg
        },
        None => {
            let cfg = Config::preset(method);
            info!("Using {:?} preset: {:#?}", method, cfg);
            cfg
        },
    };

    /*
     * Verify requirements
     */
    let apriori_ecef = ctx.rx_ecef.ok_or(Error::UndefinedAprioriPosition)?;

    let apriori = Vector3::<f64>::new(apriori_ecef.0, apriori_ecef.1, apriori_ecef.2);
    let apriori = AprioriPosition::from_ecef(apriori);
    let rx_lat_ddeg = apriori.geodetic()[0];

    if ctx.data.observation().is_none() {
        panic!("Position solver requires Observation RINEX");
    }

    let nav_data = ctx
        .data
        .brdc_navigation()
        .expect("positioning requires Navigation RINEX");

    let sp3_data = ctx.data.sp3();
    if sp3_data.is_none() {
        panic!("High precision orbits (SP3) are unfortunately mandatory at the moment..");
    }

    // print config to be used
    info!("Using solver {:?} method", method);
    info!("Using solver configuration {:#?}", cfg);

    /*
     * Warn against interpolation errors
     */
    if ctx.data.clock().is_none() {
        if ctx.data.sp3_has_clock() {
            let sp3 = ctx.data.sp3().unwrap();
            if sp3.epoch_interval >= Duration::from_seconds(300.0) {
                warn!("Interpolating clock states from low sample rate SP3 will most likely introduce errors");
            }
        }
    }

    let solver = Solver::new(
        cfg,
        apriori,
        if matches.get_flag("cggtts") {
            PVTSolutionType::TimeOnly
        } else {
            PVTSolutionType::PositionVelocityTime
        },
    );

    let rinex = ctx
        .data
        .observation()
        .unwrap_or_else(|| panic!("positioning required Observation RINEX"));

    let clocks = ClockIter::from_ctx(ctx);
    let observations = ObservationIter::from_ctx(rinex.pseudo_range());

    //if matches.get_flag("cggtts") {
    //    /* CGGTTS special opmode */
    //    let tracks = cggtts::resolve(ctx, solver, rx_lat_ddeg, matches)?;
    //    cggtts_post_process(ctx, tracks, matches)?;
    //} else {
    /* PPP */
    let pvt_solutions = ppp::resolve(solver, rx_lat_ddeg, observations, clocks);
    /* save solutions (graphs, reports..) */
    ppp_post_process(ctx, pvt_solutions, matches)?;
    //}
    Ok(())
}
