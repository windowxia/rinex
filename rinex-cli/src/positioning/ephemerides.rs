use crate::cli::Context;
use itertools::Itertools;

use rinex::{
    observation::ObservationData,
    prelude::{EpochFlag, Observable},
};

use hifitime::{Epoch, TimeScale, Unit};

use rtk::prelude::{
    Ephemerides as RTKEphemerides,
    EphemeridesIter as RTKEphemeridesIter,
};

/// Efficient Observation stream
pub struct EphemeridesIter<'a> {
    iter: Box<dyn Iterator<Item = RTKEphemerides> + 'a>,
}

impl<'a> EphemeridesIter<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        let nav = ctx.data.brdc_navigation().unwrap(); // infaillible
        Self {
            iter: Box::new(nav.ephemeris().filter_map(|(t, (_, sv, ephemeris))| {
                let toe_week = ephemeris.get_week()?;
                let toe_secs = ephemeris.kepler()?.toe;
                let toe_week_gpst = match sv.timescale() {
                    Some(TimeScale::GST) => toe_week - 1024,
                    _ => toe_week,
                };
                Some(RTKEphemerides::new(
                    *t,
                    sv,
                    (ephemeris.clock_bias, 
                    ephemeris.clock_drift, 
                    ephemeris.clock_drift_rate),
                        Epoch::from_duration(
                            (toe_week_gpst as f64) * 7.0 * Unit::Day + toe_secs * Unit::Second,
                            TimeScale::GPST,
                        ),
                ))
            }))
        }
    }
}

impl<'a> RTKEphemeridesIter for EphemeridesIter<'a> {
    fn next(&mut self) -> Option<RTKEphemerides> {
        self.iter.next()
    }
}
