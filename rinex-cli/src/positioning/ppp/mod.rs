//! PPP solver
use crate::cli::Context;

use super::{ClockIter, EphemeridesIter, IonosphereModelIter, ObservationIter, OrbitIter};

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
    iono_models: IonosphereModelIter,
) -> Vec<PVTSolution> {
    solver.resolve(ephemerides, orbits, clocks, observations, iono_models)
}
