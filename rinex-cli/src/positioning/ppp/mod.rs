//! PPP solver
use crate::cli::Context;

use super::{ClockIter, EphemeridesIter, IonosphereModelIter, ObservationIter, OrbitIter, SVInfoIter};

mod post_process;
pub use post_process::{post_process, Error as PostProcessingError};

use rtk::prelude::{PVTSolution, Solver};

pub fn resolve(
    mut solver: Solver,
    rx_lat_ddeg: f64,
    ephemerides: EphemeridesIter,
    observations: ObservationIter,
    orbits: OrbitIter,
    clocks: ClockIter,
    sv_infos: SVInfoIter,
    iono_models: IonosphereModelIter,
) -> Vec<PVTSolution> {
    solver.resolve(ephemerides, orbits, clocks, observations, sv_infos, iono_models)
}
