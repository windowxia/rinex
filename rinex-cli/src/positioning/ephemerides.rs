use crate::cli::Context;

use rinex::{
    observation::ObservationData,
    prelude::{EpochFlag, Observable},
};

use hifitime::{Epoch, TimeScale, Unit};
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
                assert!(
                    t.time_scale.is_gnss(),
                    "Only GNSS timescales expected here - invalid RINEX"
                );
                let toe_week_gpst = match sv.timescale() {
                    Some(TimeScale::GST) => toe_week - 1024,
                    _ => toe_week,
                };
                Some(Epoch::from_duration(
                    (toe_week_gpst as f64) * 7.0 * Unit::Day + toe_secs * Unit::Second,
                    TimeScale::GPST,
                ))
            })),
        }
    }
}

impl<'a> RTKEphemeridesIter for EphemeridesIter<'a> {
    fn next(&mut self) -> Option<Epoch> {
        self.iter.next()
    }
}
