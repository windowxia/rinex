use crate::cli::Context;

use rinex::prelude::SV;
use rtk::prelude::{
    SVInfo as RTKSVInfo,
    SVInfoIter as RTKSVInfoIter,
};

/// Efficient info stream
pub struct SVInfoIter<'a> {
    iter: Box<dyn Iterator<Item = RTKSVInfo> + 'a>,
}

impl<'a> SVInfoIter<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        let nav = ctx.data.brdc_navigation().unwrap(); // infaillible
        Self {
            iter: Box::new(nav.ephemeris().filter_map(|(_, (_, sv, ephemeris))| {
                let tgd = ephemeris.tgd()?;
                Some(RTKSVInfo {
                    sv,
                    tgd,
                })
            })),
        }
    }
}

impl<'a> RTKSVInfoIter for SVInfoIter<'a> {
    fn next(&mut self) -> Option<RTKSVInfo> {
        self.iter.next()
    }
}
