//! RINEX / GNSS data processing in general
use gnss_rs::prelude::{Constellation, COSPAR, DOMES, SV};
use hifitime::{Duration, Epoch};

mod token;
use token::Token;

pub mod masking;
pub use masking::{MaskFilter, MaskOperand, Masking};

pub mod resampling;
use resampling::ResamplingOps;
pub use resampling::{DecimationFilter, Resampling, ResamplingFilter};

#[derive(Debug)]
pub enum Error {
    /// Invalid [MaskOperand] description
    InvalidOperand,
    /// Invalid [MaskFilter] description
    InvalidMask,
    /// Invalid [Epoch] description
    InvalidEpoch,
    /// Invalid [Duration] description
    InvalidDuration,
    /// Invalid Elevation Angle description
    InvalidElevation,
    /// Invalid Azimuth Angle description
    InvalidAzimuth,
    /// Invalid [Constellation] description
    InvalidConstellation,
    /// Invalid [DOMES] site description
    InvalidDOMES,
    /// Invalid frequency description
    InvalidFrequency,
    /// Invalid Filter description
    InvalidFilter,
    /// Invalid Token description
    InvalidToken,
    /// Invalid R Decimation Filter
    InvalidDecimationRatio,
}

/// Supported Filter types
#[derive(Debug, PartialEq)]
pub enum Filter {
    /// Mask filter to retain or discard data subsets
    Mask(MaskFilter),
    /// Resampling filter to resample entire datasets or subsets
    Resampling(ResamplingFilter),
}

impl std::str::FromStr for Filter {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("d:") {
            let filter = ResamplingFilter::parse_decimation(&s[2..])?;
            Ok(Self::Resampling(filter))
        } else {
            let filter = MaskFilter::from_str(&s[2..])?;
            Ok(Self::Mask(filter))
        }
    }
}

/// Most structures need to implement the Preprocessing Trait,
/// to rework or adapt Self prior further analysis
pub trait Preprocessing: Masking + Resampling {
    /// Apply [Filter] to self returning a new Self.
    /// Use [filter] to rework data set prior further analysis.
    fn filter(&self, f: &Filter) -> Self
    where
        Self: Sized;
    /// Apply [Filter] to mutable self, reworking self in place.
    /// Use [filter_mut] to rework data set prior further analysis.
    fn filter_mut(&mut self, f: &Filter) {
        match f {
            Filter::Mask(m) => self.mask_mut(m),
            Filter::Resampling(r) => self.resample_mut(r),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{DecimationFilter, Filter, ResamplingFilter, ResamplingOps};
    use crate::processing::MaskFilter;
    use crate::processing::MaskOperand;
    use crate::processing::Token;
    use hifitime::Epoch;
    use std::str::FromStr;
    #[test]
    fn decimation_simple_parsing() {
        for (desc, expected) in [
            ("d:3", DecimationFilter::ByRatio(3)),
            ("d:10", DecimationFilter::ByRatio(10)),
            ("d:30", DecimationFilter::ByRatio(30)),
            ("d:50", DecimationFilter::ByRatio(50)),
        ] {
            let decim = Filter::from_str(desc);
            assert!(decim.is_ok(), "failed to parse decim filter \"{}\"", desc);
            let decim = decim.unwrap();
            assert_eq!(
                decim,
                Filter::Resampling(ResamplingFilter {
                    mask: None,
                    ops: ResamplingOps::Decimation(expected),
                })
            );
        }
    }
    #[test]
    fn decimation_complex_parsing() {
        for (desc, mask, expected) in [
            (
                "d:sta=ESBCDNK*3",
                MaskFilter {
                    operand: MaskOperand::Equals,
                    token: Token::Stations(vec!["ESBCDNK".to_string()]),
                },
                DecimationFilter::ByRatio(3),
            ),
            (
                "d:t>2020-01-01T00:00:00 GPST*5",
                MaskFilter {
                    operand: MaskOperand::GreaterThan,
                    token: Token::Epoch(Epoch::from_str("2020-01-01T00:00:00 GPST").unwrap()),
                },
                DecimationFilter::ByRatio(5),
            ),
        ] {
            let decim = Filter::from_str(desc);
            assert!(
                decim.is_ok(),
                "failed to parse decim filter \"{}\": {:?}",
                desc,
                decim.err().unwrap()
            );
            let decim = decim.unwrap();
            assert_eq!(
                decim,
                Filter::Resampling(ResamplingFilter {
                    mask: Some(mask),
                    ops: ResamplingOps::Decimation(expected),
                })
            );
        }
    }
}
