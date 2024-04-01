use crate::cli::Context;
use std::fs::read_to_string;
use thiserror::Error;

mod clock;
mod ephemerides;
mod ionosphere;
mod observation;
mod orbit;

mod ppp; // precise point positioning
use ppp::post_process as ppp_post_process;
use ppp::PostProcessingError as PPPPostProcessingError;

//mod cggtts; // CGGTTS special solver
//use cggtts::post_process as cggtts_post_process;
//use cggtts::PostProcessingError as CGGTTSPostProcessingError;

use clap::ArgMatches;
use gnss::prelude::{Constellation, SV};

use rinex::{
    //navigation::Ephemeris,
    prelude::{EpochFlag, Observable, Rinex},
};

use std::collections::{BTreeMap, HashMap};

use rtk::prelude::{
    AprioriPosition, Config, Duration, Epoch, Method, PVTSolutionType, Solver, Vector3,
};

use map_3d::{ecef2geodetic, rad2deg, Ellipsoid};

pub use clock::ClockIter;
pub use ephemerides::EphemeridesIter;
pub use ionosphere::IonosphereModelIter;
pub use observation::ObservationIter;
pub use orbit::OrbitIter;

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

pub fn precise_positioning(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    /* Resolution method */
    let method = match matches.get_flag("spp") {
        true => Method::SPP,
        false => panic!("PPP is not supported at the moment"),
    };

    let cfg = match matches.get_one::<String>("cfg") {
        Some(fp) => {
            let content = read_to_string(fp)
                .unwrap_or_else(|_| panic!("failed to read configuration: permission denied"));
            let cfg = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("failed to parse configuration: invalid content"));
            info!("Using custom configuration: {:#?}", cfg);
            cfg
        },
        None => {
            let cfg = Config::preset(method);
            cfg
        },
    };

    /*
     * Verify requirements
     */
    let apriori_ecef = ctx.rx_ecef.ok_or(Error::UndefinedAprioriPosition)?;

    let apriori = Vector3::<f64>::new(apriori_ecef.0, apriori_ecef.1, apriori_ecef.2);
    let apriori = AprioriPosition::from_ecef(apriori);
    let rx_lat_ddeg = apriori.geodetic_ddeg()[0];

    if ctx.data.observation().is_none() {
        panic!("Position solver requires Observation RINEX");
    }

    let nav_data = ctx
        .data
        .brdc_navigation()
        .expect("positioning requires Navigation RINEX");

    // print config to be used
    info!("Using {:?} method", method);
    info!("Using configuration {:#?}", cfg);

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
        apriori.clone(),
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
    let orbits = OrbitIter::from_ctx(ctx, &apriori);
    let ephemerides = EphemeridesIter::from_ctx(ctx);
    let iono_models = IonosphereModelIter::from_ctx(ctx);
    let observations = ObservationIter::from_ctx(rinex.pseudo_range());

    //if matches.get_flag("cggtts") {
    //    /* CGGTTS special opmode */
    //    let tracks = cggtts::resolve(ctx, solver, rx_lat_ddeg, matches)?;
    //    cggtts_post_process(ctx, tracks, matches)?;
    //} else {
    /* PPP */
    let pvt_solutions = ppp::resolve(
        solver,
        rx_lat_ddeg,
        ephemerides,
        observations,
        orbits,
        clocks,
        iono_models,
    );
    /* save solutions (graphs, reports..) */
    ppp_post_process(ctx, pvt_solutions, matches)?;
    //}
    Ok(())
}
