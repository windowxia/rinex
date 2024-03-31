//! PPP solver
use crate::cli::Context;
use crate::positioning::{bd_model, kb_model, ng_model, tropo_components};

use rinex::{
    carrier::Carrier,
    navigation::Ephemeris,
    prelude::{Duration, SV},
};

use super::{ClockIter, ObservationIter, OrbitIter};

mod post_process;
pub use post_process::{post_process, Error as PostProcessingError};

use rtk::prelude::{
    Epoch, IonosphericBias, Observation, PVTSolution, PVTSolutionType, Solver, TroposphericBias,
    Vector3,
};

pub fn resolve(
    mut solver: Solver,
    rx_lat_ddeg: f64,
    observations: ObservationIter,
    orbits: OrbitIter,
    clocks: ClockIter,
) -> Vec<PVTSolution> {
    //for ((t, flag), (_clk, vehicles)) in obs_data.observation() {
    //    let mut candidates = Vec::<Candidate>::with_capacity(4);

    //    if !flag.is_ok() {
    //        /* we only consider _valid_ epochs" */
    //        continue;
    //    }

    //    // /*
    //    //  * store possibly provided clk state estimator,
    //    //  * so we can compare ours to this one later
    //    //  */
    //    // if let Some(clk) = clk {
    //    //     provided_clk.insert(*t, *clk);
    //    // }

    //    for (sv, observations) in vehicles {
    //        let sv_eph = nav_data.sv_ephemeris(*sv, *t);
    //        if sv_eph.is_none() {
    //            debug!("{:?} ({}) : undetermined ephemeris", t, sv);
    //            continue; // can't proceed further
    //        }

    //        // determine TOE
    //        let (toe, _sv_eph) = sv_eph.unwrap();

    //        /*
    //         * Clock state
    //         *   1. Prefer CLK product: best quality
    //         *   2. Prefer SP3 product: most likely incompatible with very precise PPP
    //         *   3. BRDC Radio last option: always feasible but most likely very noisy/wrong
    //         */
    //        let clock_state = if let Some(clk) = clk_data {
    //            if let Some((_, profile)) = clk.precise_sv_clock_interpolate(*t, *sv) {
    //                (
    //                    profile.bias,
    //                    profile.drift.unwrap_or(0.0),
    //                    profile.drift_change.unwrap_or(0.0),
    //                )
    //            } else {
    //                /*
    //                 * interpolation failure.
    //                 * Do not interpolate other products: SV will not be presented.
    //                 */
    //                continue;
    //            }
    //        } else if sp3_has_clock {
    //            if let Some(sp3) = sp3_data {
    //                if let Some(bias) = sp3.sv_clock_interpolate(*t, *sv) {
    //                    // FIXME:
    //                    // slightly rework SP3 to expose drift + driftr better
    //                    (bias, 0.0_f64, 0.0_f64)
    //                } else {
    //                    /*
    //                     * interpolation failure.
    //                     * Do not interpolate other products: SV will not be presented.
    //                     */
    //                    continue;
    //                }
    //            } else {
    //                // FIXME: BRDC interpolation
    //                continue;
    //            }
    //        } else {
    //            // FIXME: BRDC interpolation
    //            continue;
    //        };

    //        // determine clock correction
    //        let clock_corr = Ephemeris::sv_clock_corr(*sv, clock_state, *t, toe);

    //        let mut codes = Vec::<Observation>::new();
    //        let mut phases = Vec::<Observation>::new();
    //        let mut dopplers = Vec::<Observation>::new();

    //        for (observable, data) in observations {
    //            if let Ok(carrier) = Carrier::from_observable(sv.constellation, observable) {
    //                let frequency = carrier.frequency();

    //                if observable.is_pseudorange_observable() {
    //                    codes.push(Observation {
    //                        frequency,
    //                        snr: { data.snr.map(|snr| snr.into()) },
    //                        value: data.obs,
    //                    });
    //                } else if observable.is_phase_observable() {
    //                    phases.push(Observation {
    //                        frequency,
    //                        snr: { data.snr.map(|snr| snr.into()) },
    //                        value: data.obs,
    //                    });
    //                } else if observable.is_doppler_observable() {
    //                    dopplers.push(Observation {
    //                        frequency,
    //                        snr: { data.snr.map(|snr| snr.into()) },
    //                        value: data.obs,
    //                    });
    //                }
    //            }
    //        }

    //        let clock_state = Vector3::new(clock_state.0, clock_state.1, clock_state.2);

    //        if let Ok(candidate) = Candidate::new(
    //            *sv,
    //            *t,
    //            clock_state,
    //            clock_corr,
    //            codes.clone(),
    //            phases.clone(),
    //            dopplers.clone(),
    //        ) {
    //            candidates.push(candidate);
    //        } else {
    //            warn!("{:?}: failed to form {} candidate", t, sv);
    //        }
    //    }

    //    // grab possible tropo components
    //    let zwd_zdd = tropo_components(meteo_data, *t, rx_lat_ddeg);

    //    let iono_bias = IonosphericBias {
    //        kb_model: kb_model(nav_data, *t),
    //        bd_model: bd_model(nav_data, *t),
    //        ng_model: ng_model(nav_data, *t),
    //        stec_meas: None, //TODO
    //    };

    //    let tropo_bias = TroposphericBias {
    //        total: None, //TODO
    //        zwd_zdd,
    //    };
    match solver.resolve(orbits, clocks, observations) {
        Ok(solutions) => solutions,
        Err(e) => panic!("solver error: {:?}", e),
    }
}
