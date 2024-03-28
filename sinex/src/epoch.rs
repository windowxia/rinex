use hifitime::{Epoch, Unit};
use thiserror::Error;

/// Epoch (datetime) parsing error
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid format")]
    InvalidFormat,
    #[error("invalid epoch")]
    InvalidEpoch(#[from] hifitime::Errors),
    #[error("failed to parse subseconds")]
    SecondParsing(#[from] std::num::ParseFloatError),
}

/// Parses SINEX standardized Epoch
pub(crate) fn parse_epoch(content: &str) -> Result<Epoch, Error> {
    if content.len() < 10 {
        return Err(Error::InvalidFormat);
    }
    let e = Epoch::from_format_str(content, "%Y:%j")?;
    let secs = content[9..].parse::<f64>()?;
    Ok(e + secs * Unit::Second)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parsing() {
        let epoch = parse_epoch("2022:021:20823");
        assert!(epoch.is_ok());

        let epoch = parse_epoch("2022:009:00000");
        assert!(epoch.is_ok());
    }
}
