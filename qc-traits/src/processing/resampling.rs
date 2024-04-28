use super::{Error, Token};
use crate::processing::MaskFilter;
use gnss_rs::prelude::{Constellation, COSPAR, DOMES, SV};
use hifitime::{Duration, Epoch};
use std::str::FromStr;

/// Supported Decimation Filters
#[derive(Debug, PartialEq)]
pub enum DecimationFilter {
    /// Decimate by a given factor
    ByRatio(u32),
}

impl Default for DecimationFilter {
    fn default() -> Self {
        Self::ByRatio(0)
    }
}

impl std::str::FromStr for DecimationFilter {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = s
            .trim()
            .parse::<u32>()
            .map_err(|_| Error::InvalidDecimationRatio)?;
        Ok(Self::ByRatio(r))
    }
}

/// Supported Resampling Operations
#[derive(Debug, PartialEq)]
pub enum ResamplingOps {
    /// Decimate to reduce data rate
    Decimation(DecimationFilter),
}

/// Resampling Filter to resample entire set or subsets
#[derive(Debug, PartialEq)]
pub struct ResamplingFilter {
    /// Possible [MaskFilter]
    pub mask: Option<MaskFilter>,
    /// [ResamplingOps] to describe how to resample
    pub ops: ResamplingOps,
}

impl ResamplingFilter {
    pub(crate) fn parse_decimation(s: &str) -> Result<Self, Error> {
        let mut mask = Option::<MaskFilter>::None;
        let mut decim = DecimationFilter::default();
        match s.find('*') {
            Some(offset) => {
                let mask = MaskFilter::from_str(s[..offset].trim())?;
                let decim = DecimationFilter::from_str(s[offset + 1..].trim())?;
                Ok(Self {
                    mask: Some(mask),
                    ops: ResamplingOps::Decimation(decim),
                })
            },
            None => {
                let decim = DecimationFilter::from_str(s.trim())?;
                Ok(Self {
                    mask: None,
                    ops: ResamplingOps::Decimation(decim),
                })
            },
        }
    }
}

/// Resampling Trait, to resample entire datasets or subsets
pub trait Resampling {
    /// Applies [ResamplingFilter] returning a copied Self.
    fn resample(&self, resamp: &ResamplingFilter) -> Self
    where
        Self: Sized;
    /// Applies [ResamplingFilter] in place with mutable access.
    fn resample_mut(&mut self, resamp: &ResamplingFilter);
}
