//! RINEX / GNSS data processing in general
use gnss_rs::prelude::{Constellation, COSPAR, DOMES, SV};
use hifitime::{Duration, Epoch};

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
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub enum MaskOperand {
    /// Equality operator, described by '='
    #[default]
    Equals = 0,
    /// Inequality operator, described by "!="
    NotEquals = 1,
    /// Greater than, described by '>'
    GreaterThan = 2,
    /// Greater Equals, described by ">="
    GreaterEquals = 3,
    /// Lower than, described by '<'
    LowerThan = 4,
    /// Lower Equals, described by "<="
    LowerEquals = 5,
}

impl std::fmt::Display for MaskOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Equals => write!(f, "{}", '='),
            Self::GreaterThan => write!(f, "{}", '>'),
            Self::LowerThan => write!(f, "{}", '<'),
            Self::NotEquals => write!(f, "{}", "!="),
            Self::GreaterEquals => write!(f, "{}", ">="),
            Self::LowerEquals => write!(f, "{}", "<="),
        }
    }
}

impl std::str::FromStr for MaskOperand {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('=') {
            Ok(Self::Equals)
        } else if s.starts_with(">=") {
            Ok(Self::GreaterEquals)
        } else if s.starts_with("<=") {
            Ok(Self::LowerEquals)
        } else if s.starts_with("!=") {
            Ok(Self::NotEquals)
        } else if s.starts_with('>') {
            Ok(Self::GreaterThan)
        } else if s.starts_with('<') {
            Ok(Self::LowerThan)
        } else {
            Err(Error::InvalidOperand)
        }
    }
}

impl MaskOperand {
    const fn formatted_len(&self) -> usize {
        match &self {
            Self::Equals | Self::GreaterThan | Self::LowerThan => 1,
            Self::NotEquals | Self::LowerEquals | Self::GreaterEquals => 2,
        }
    }
}

impl std::ops::Not for MaskOperand {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Self::Equals => Self::NotEquals,
            Self::NotEquals => Self::Equals,
            Self::GreaterEquals => Self::LowerEquals,
            Self::GreaterThan => Self::LowerThan,
            Self::LowerThan => Self::GreaterThan,
            Self::LowerEquals => Self::GreaterEquals,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    /// Epoch
    Epoch(Epoch),
    /// Duration
    Duration(Duration),
    /// SV Elevation angle in deg°
    Elevation(f64),
    /// SV Azimuth angle in deg°
    Azimuth(f64),
    /// List of GNSS signal frequencies in [Hz]
    Frequencies(Vec<f64>),
    /// List of Satellite Vehicles
    SV(Vec<SV>),
    /// LIst of Satellie Vehicles by COSPAR number
    COSPAR(Vec<COSPAR>),
    /// List of GNSS observables (standard RINEX codes)
    Observables(Vec<String>),
    /// List of GNSS Constellations
    Constellations(Vec<Constellation>),
    /// List of Stations by DOMES codes
    DOMES(Vec<DOMES>),
    /// List of Stations or Agencies by name
    Stations(Vec<String>),
}

/// Mask filter to retain or discard data subsets
#[derive(Debug, PartialEq)]
pub struct MaskFilter {
    /// [Token]
    pub token: Token,
    /// [MaskOperand] to describe how to handle [FilterItem]
    pub operand: MaskOperand,
}

impl std::str::FromStr for MaskFilter {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("dt") {
            let operand = MaskOperand::from_str(&s[2..])?;
            let offset = operand.formatted_len() + 2;
            Ok(Self {
                operand,
                token: Token::Duration(
                    Duration::from_str(&s[offset..].trim()).map_err(|_| Error::InvalidDuration)?,
                ),
            })
        } else if s.starts_with('t') {
            let operand = MaskOperand::from_str(&s[1..])?;
            let offset = operand.formatted_len() + 1;
            Ok(Self {
                operand,
                token: Token::Epoch(
                    Epoch::from_str(&s[offset..].trim()).map_err(|_| Error::InvalidEpoch)?,
                ),
            })
        } else if s.starts_with('e') {
            let operand = MaskOperand::from_str(&s[1..])?;
            let offset = operand.formatted_len() + 1;
            let elevation =
                f64::from_str(&s[offset..].trim()).map_err(|_| Error::InvalidElevation)?;
            if elevation < 0.0 || elevation > 90.0 {
                return Err(Error::InvalidElevation);
            }
            Ok(Self {
                operand,
                token: Token::Elevation(elevation),
            })
        } else if s.starts_with("az") {
            let operand = MaskOperand::from_str(&s[2..])?;
            let offset = operand.formatted_len() + 2;
            let azimuth = f64::from_str(&s[offset..].trim()).map_err(|_| Error::InvalidAzimuth)?;
            if azimuth < 0.0 || azimuth > 360.0 {
                return Err(Error::InvalidAzimuth);
            }
            Ok(Self {
                operand,
                token: Token::Azimuth(azimuth),
            })
        } else if s.starts_with('c') {
            let operand = MaskOperand::from_str(&s[1..])?;
            let offset = operand.formatted_len() + 1;
            let constellations = s[offset..]
                .trim()
                .split(',')
                .filter_map(|c| {
                    if let Ok(c) = Constellation::from_str(c) {
                        Some(c)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if constellations.len() == 0 {
                return Err(Error::InvalidConstellation);
            }
            Ok(Self {
                operand,
                token: Token::Constellations(constellations),
            })
        } else if s.starts_with('f') {
            let operand = MaskOperand::from_str(&s[1..])?;
            let offset = operand.formatted_len() + 1;
            let freqz = s[offset..]
                .trim()
                .split(',')
                .filter_map(|c| {
                    if let Ok(f) = f64::from_str(c) {
                        Some(f)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            Ok(Self {
                operand,
                token: Token::Frequencies(freqz),
            })
        } else if s.starts_with('o') {
            let operand = MaskOperand::from_str(&s[1..])?;
            let offset = operand.formatted_len() + 1;
            let observables = s[offset..]
                .trim()
                .split(',')
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
            Ok(Self {
                operand,
                token: Token::Observables(observables),
            })
        } else {
            Err(Error::InvalidMask)
        }
    }
}

/// Supported Filter types
pub enum Filter {
    /// Mask filter to retain or discard data subsets
    Mask(MaskFilter),
}

/// Masking Trait, to retain or discard data subsets
pub trait Masking {
    /// Applies [MaskFilter] returning a copied Self.
    fn mask(&self, mask: &MaskFilter) -> Self
    where
        Self: Sized;
    /// Applies [MaskFilter] in place with mutable access.
    fn mask_mut(&mut self, mask: &MaskFilter);
}

/// Most structures need to implement the Preprocessing Trait,
/// to rework or adapt Self prior further analysis
pub trait Preprocessing: Masking {
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
        }
    }
}

#[cfg(test)]
mod test {
    use crate::processing::{MaskFilter, MaskOperand, Token};
    use hifitime::{Duration, Epoch};
    use std::str::FromStr;
    #[test]
    fn operand_parsing() {
        for (desc, operand, not) in [
            ("=", MaskOperand::Equals, MaskOperand::NotEquals),
            ("!=", MaskOperand::NotEquals, MaskOperand::Equals),
            ("<", MaskOperand::LowerThan, MaskOperand::GreaterEquals),
            ("<=", MaskOperand::LowerEquals, MaskOperand::GreaterThan),
            (">", MaskOperand::GreaterThan, MaskOperand::LowerEquals),
            (">=", MaskOperand::GreaterEquals, MaskOperand::LowerThan),
        ] {
            let op = MaskOperand::from_str(desc).unwrap();
            assert_eq!(op, operand, "failed to parse \"{}\" operand", desc);
            assert_eq!(!op, not);
            assert_eq!(desc, operand.to_string(), "operand::to_string reciprocal");
        }
    }
    #[test]
    fn dt_mask_parsing() {
        for (desc, mask) in [
            (
                "dt=1 hour",
                MaskFilter {
                    operand: MaskOperand::Equals,
                    token: Token::Duration(Duration::from_hours(1.0)),
                },
            ),
            (
                "dt=30 s",
                MaskFilter {
                    operand: MaskOperand::Equals,
                    token: Token::Duration(Duration::from_seconds(30.0)),
                },
            ),
            (
                "dt>30 s",
                MaskFilter {
                    operand: MaskOperand::GreaterThan,
                    token: Token::Duration(Duration::from_seconds(30.0)),
                },
            ),
            (
                "dt<1 min",
                MaskFilter {
                    operand: MaskOperand::LowerThan,
                    token: Token::Duration(Duration::from_seconds(60.0)),
                },
            ),
            (
                "dt<=1 min",
                MaskFilter {
                    operand: MaskOperand::LowerEquals,
                    token: Token::Duration(Duration::from_seconds(60.0)),
                },
            ),
        ] {
            let parsed = MaskFilter::from_str(desc).unwrap();
            assert_eq!(parsed, mask, "failed to parse \"{}\"", desc);
        }
    }
    #[test]
    fn epoch_mask_parsing() {
        for (desc, mask) in [
            (
                "t=2020-01-01T00:00:00 UTC",
                MaskFilter {
                    operand: MaskOperand::Equals,
                    token: Token::Epoch(Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap()),
                },
            ),
            (
                "t<JD 2452312.500372511 TAI",
                MaskFilter {
                    operand: MaskOperand::LowerThan,
                    token: Token::Epoch(Epoch::from_str("JD 2452312.500372511 TAI").unwrap()),
                },
            ),
            (
                "t>2030-01-01T01:01:01 GPST",
                MaskFilter {
                    operand: MaskOperand::GreaterThan,
                    token: Token::Epoch(Epoch::from_str("2030-01-01T01:01:01 GPST").unwrap()),
                },
            ),
        ] {
            let parsed = MaskFilter::from_str(desc).unwrap();
            assert_eq!(parsed, mask, "failed to parse \"{}\"", desc);
        }
    }
    #[test]
    fn elevation_mask_parsing() {
        for (desc, mask) in [(
            "e> 30",
            MaskFilter {
                operand: MaskOperand::GreaterThan,
                token: Token::Elevation(30.0),
            },
        )] {
            let parsed = MaskFilter::from_str(desc).unwrap();
            assert_eq!(parsed, mask, "failed to parse \"{}\"", desc);
        }
    }
    #[test]
    fn azimuth_mask_parsing() {
        for (desc, mask) in [(
            "az< 10",
            MaskFilter {
                operand: MaskOperand::LowerThan,
                token: Token::Azimuth(10.0),
            },
        )] {
            let parsed = MaskFilter::from_str(desc).unwrap();
            assert_eq!(parsed, mask, "failed to parse \"{}\"", desc);
        }
    }
    #[test]
    fn observable_mask_parsing() {
        for (desc, mask) in [(
            "o=L1,L2",
            MaskFilter {
                operand: MaskOperand::Equals,
                token: Token::Observables(vec!["L1".to_string(), "L2".to_string()]),
            },
        )] {
            let parsed = MaskFilter::from_str(desc).unwrap();
            assert_eq!(parsed, mask, "failed to parse \"{}\"", desc);
        }
    }
}
