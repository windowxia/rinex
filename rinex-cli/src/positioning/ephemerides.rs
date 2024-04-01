use crate::cli::Context;
use hifitime::Epoch;

use rinex::{
    observation::ObservationData,
    prelude::{EpochFlag, Observable},
};

use rtk::prelude::EphemeridesIter as RTKEphemeridesIter;

/// Efficient Observation stream
pub struct EphemeridesIter<'a> {
    iter: Box<dyn Iterator<Item = Epoch> + 'a>,
}

impl<'a> EphemeridesIter<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        let nav = ctx.data.brdc_navigation().unwrap(); // infaillible
        Self {
            iter: Box::new(nav.ephemeris().filter_map(|(t, (_, sv, ephemeris))| {
                let toe_week = ephemeris.get_week()?;
                let toe_secs = ephemeris.kepler()?.toe;
                Some(Epoch::default()) //TODO
            })),
        }
    }
}

impl<'a> RTKEphemeridesIter for EphemeridesIter<'a> {
    fn next(&mut self) -> Option<Epoch> {
        self.iter.next()
    }
}
