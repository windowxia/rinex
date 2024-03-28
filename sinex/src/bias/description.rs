use crate::bias;
use gnss::prelude::Constellation;
use std::collections::HashMap;

use hifitime::TimeScale;

#[derive(Debug, Clone, Default)]
pub struct Description {
    /// Observation Sampling: sampling interval in seconds
    pub sampling: Option<u32>,
    /// Parameter Spacing: spacing interval in seconds,
    /// used for parameter representation
    pub spacing: Option<u32>,
    /// Method used to generate the bias results
    pub method: Option<bias::Method>,
    /// See [bias::header::BiasMode]
    pub bias_mode: bias::header::BiasMode,
    /// TimeScale
    pub timescale: TimeScale,
    /// Receiver clock reference GNSS
    pub rcvr_clock_ref: Option<Constellation>,
    /// Satellite clock reference observables:
    /// list of observable codes (standard 3 letter codes),
    /// for each GNSS in this file.
    /// Must be provided if associated bias results are consistent
    /// with the ionosphere free LC, otherwise, these might be missing
    pub sat_clock_ref: HashMap<Constellation, Vec<String>>,
}

impl Description {
    pub(crate) fn with_sampling(&self, sampling: u32) -> Self {
        let mut s = self.clone();
        s.sampling = Some(sampling);
        s
    }
    pub(crate) fn with_spacing(&self, spacing: u32) -> Self {
        let mut s = self.clone();
        s.spacing = Some(spacing);
        s
    }
    pub(crate) fn with_method(&self, method: bias::Method) -> Self {
        let mut s = self.clone();
        s.method = Some(method);
        s
    }
    pub(crate) fn with_bias_mode(&self, mode: bias::header::BiasMode) -> Self {
        let mut s = self.clone();
        s.bias_mode = mode;
        s
    }
    pub(crate) fn with_timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.timescale = ts;
        s
    }
    pub(crate) fn with_rcvr_clock_ref(&self, clock_ref: Constellation) -> Self {
        let mut s = self.clone();
        s.rcvr_clock_ref = Some(clock_ref);
        s
    }
    pub fn with_sat_clock_ref(&self, constell: Constellation, observable: &str) -> Self {
        let mut s = self.clone();
        if let Some(codes) = s.sat_clock_ref.get_mut(&constell) {
            codes.push(observable.to_string());
        } else {
            s.sat_clock_ref
                .insert(constell, vec![observable.to_string()]);
        }
        s
    }
}
