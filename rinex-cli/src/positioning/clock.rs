use crate::cli::Context;

use rtk::prelude::{Clock, ClockIter as RTKClockIter, Epoch, SV};

/// Efficient Clock state stream
pub struct ClockIter<'a> {
    iter: Box<dyn Iterator<Item = (Epoch, SV, f64, Option<f64>, Option<f64>)> + 'a>,
}

impl<'a> ClockIter<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        let sp3_has_clk = ctx.data.sp3_has_clock();
        Self {
            iter: if let Some(clk) = ctx.data.clock() {
                Box::new(clk.precise_sv_clock().map(|(t, sv, _, profile)| {
                    (t, sv, profile.bias, profile.drift, profile.drift_change)
                }))
            } else if sp3_has_clk {
                let sp3 = ctx.data.sp3().unwrap();
                Box::new(sp3.sv_clock().map(|(t, sv, offset)| {
                    (t, sv, offset, None, None) //TODO: SP3 drift + drift/r
                }))
            } else {
                let nav = ctx.data.brdc_navigation().unwrap(); // infaillible
                Box::new(nav.sv_clock().map(|(t, sv, (offset, drift, driftr))| {
                    (t, sv, offset, Some(drift), Some(driftr))
                }))
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
