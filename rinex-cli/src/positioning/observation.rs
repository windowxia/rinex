use crate::cli::Context;

use rinex::{
    carrier::Carrier,
    observation::ObservationData,
    prelude::{EpochFlag, Observable, Rinex},
};

use rtk::prelude::{Epoch, Observation, ObservationIter as RTKObservationIter, SV};

/// Efficient Observation stream
pub struct ObservationIter<'a> {
    iter: Box<dyn Iterator<Item = (Epoch, SV, f64, f64)> + 'a>,
}

impl<'a> ObservationIter<'a> {
    pub fn from_ctx(
        pseudo_range: Box<dyn Iterator<Item = (Epoch, EpochFlag, SV, &'a Observable, f64)> + 'a>,
    ) -> Self {
        Self {
            //TODO : Prefer high precision codes when that is feasible !
            iter: Box::new(pseudo_range.filter_map(|(e, flag, sv, observable, data)| {
                if flag.is_ok() {
                    if let Ok(carrier) = Carrier::from_observable(sv.constellation, observable) {
                        Some((e, sv, carrier.frequency(), data))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })),
        }
    }
}

impl<'a> RTKObservationIter for ObservationIter<'a> {
    fn next(&mut self) -> Option<Observation> {
        self.iter
            .next()
            .map(|(t, sv, freq_hz, data)| Observation::new(sv, t, data, freq_hz, None))
    }
}
