use super::{Error, Token};
use gnss_rs::prelude::{Constellation, COSPAR, DOMES, SV};
use hifitime::{Duration, Epoch};

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
            let token = Token::parse_duration(&s[offset..])?;
            Ok(Self { token, operand })
        } else if s.starts_with("sta") {
            let operand = MaskOperand::from_str(&s[3..])?;
            let offset = operand.formatted_len() + 3;
            let token = Token::parse_stations(&s[offset..])?;
            Ok(Self { token, operand })
        } else if s.starts_with("dom") {
            let operand = MaskOperand::from_str(&s[3..])?;
            let offset = operand.formatted_len() + 3;
            let token = Token::parse_domes_sites(&s[offset..])?;
            Ok(Self { token, operand })
        } else if s.starts_with('t') {
            let operand = MaskOperand::from_str(&s[1..])?;
            let offset = operand.formatted_len() + 1;
            let token = Token::parse_epoch(&s[offset..])?;
            Ok(Self { token, operand })
        } else if s.starts_with('e') {
            let operand = MaskOperand::from_str(&s[1..])?;
            let offset = operand.formatted_len() + 1;
            let token = Token::parse_elevation(&s[offset..])?;
            Ok(Self { token, operand })
        } else if s.starts_with("az") {
            let operand = MaskOperand::from_str(&s[2..])?;
            let offset = operand.formatted_len() + 2;
            let token = Token::parse_azimuth(&s[offset..])?;
            Ok(Self { token, operand })
        } else if s.starts_with('c') {
            let operand = MaskOperand::from_str(&s[1..])?;
            let offset = operand.formatted_len() + 1;
            let token = Token::parse_constellations(&s[offset..])?;
            Ok(Self { token, operand })
        } else if s.starts_with('f') {
            let operand = MaskOperand::from_str(&s[1..])?;
            let offset = operand.formatted_len() + 1;
            let token = Token::parse_frequencies(&s[offset..])?;
            Ok(Self { token, operand })
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

/// Masking Trait, to retain or discard data subsets
pub trait Masking {
    /// Applies [MaskFilter] returning a copied Self.
    fn mask(&self, mask: &MaskFilter) -> Self
    where
        Self: Sized;
    /// Applies [MaskFilter] in place with mutable access.
    fn mask_mut(&mut self, mask: &MaskFilter);
}

#[cfg(test)]
mod test {
    use super::{MaskFilter, MaskOperand};
    use crate::processing::Token;
    use gnss_rs::prelude::{DomesTrackingPoint, DOMES};
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
    #[test]
    fn station_mask_parsing() {
        for (desc, mask) in [(
            "sta=ESBCDNK",
            MaskFilter {
                operand: MaskOperand::Equals,
                token: Token::Stations(vec!["ESBCDNK".to_string()]),
            },
        )] {
            let parsed = MaskFilter::from_str(desc).unwrap();
            assert_eq!(parsed, mask, "failed to parse \"{}\"", desc);
        }
    }
    #[test]
    fn domes_mask_parsing() {
        for (desc, mask) in [(
            "dom=10002M006",
            MaskFilter {
                operand: MaskOperand::Equals,
                token: Token::DOMES(vec![DOMES {
                    area: 100,
                    site: 2,
                    sequential: 6,
                    point: DomesTrackingPoint::Monument,
                }]),
            },
        )] {
            let parsed = MaskFilter::from_str(desc).unwrap();
            assert_eq!(parsed, mask, "failed to parse \"{}\"", desc);
        }
    }
}
