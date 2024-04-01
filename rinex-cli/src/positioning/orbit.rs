use crate::cli::Context;

use rtk::prelude::{AprioriPosition, Orbit, OrbitIter as RTKOrbitIter};

/// Efficient Orbit stream
pub struct OrbitIter<'a> {
    iter: Box<dyn Iterator<Item = Orbit> + 'a>,
}

impl<'a> OrbitIter<'a> {
    pub fn from_ctx(ctx: &'a Context, apriori: &'a AprioriPosition) -> Self {
        Self {
            iter: if let Some(sp3) = ctx.data.sp3() {
                Box::new(sp3.sv_position().map(|(t, sv, pos)| {
                    Orbit::new(
                        sv,
                        t,
                        (pos.0 * 1.0E3, pos.1 * 1.0E3, pos.2 * 1.0E3),
                        apriori,
                    )
                }))
            } else {
                //let nav = ctx.data.brdc_navigation().unwrap(); // infaillible
                panic!("SP3 (High precision Orbits) are unfortunately required at the moment");
            },
        }
    }
}

impl<'a> RTKOrbitIter for OrbitIter<'a> {
    fn next(&mut self) -> Option<Orbit> {
        self.iter.next()
    }
}
