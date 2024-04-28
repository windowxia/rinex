use super::Error;
use gnss_rs::prelude::{Constellation, COSPAR, DOMES, SV};
use hifitime::{Duration, Epoch};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum MaskToken {
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
    /// SNR value [dB]
    SNR(f64),
    /// List of Satellite Vehicles
    SV(Vec<SV>),
    /// List of Satellie Vehicles by COSPAR number
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

impl MaskToken {
    pub fn parse_epoch(s: &str) -> Result<Self, Error> {
        Ok(Self::Epoch(
            Epoch::from_str(s.trim()).map_err(|_| Error::InvalidEpoch)?,
        ))
    }
    pub fn parse_duration(s: &str) -> Result<Self, Error> {
        Ok(Self::Duration(
            Duration::from_str(s.trim()).map_err(|_| Error::InvalidDuration)?,
        ))
    }
    pub fn parse_domes_sites(s: &str) -> Result<Self, Error> {
        let sites = s
            .trim()
            .split(',')
            .filter_map(|dom| {
                if let Ok(dom) = DOMES::from_str(dom.trim()) {
                    Some(dom)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if sites.is_empty() {
            return Err(Error::InvalidDOMES);
        }
        Ok(Self::DOMES(sites))
    }
    pub fn parse_stations(s: &str) -> Result<Self, Error> {
        Ok(Self::Stations(
            s.trim()
                .split(',')
                .map(|sta| sta.trim().to_string())
                .collect::<Vec<_>>(),
        ))
    }
    pub fn parse_elevation(s: &str) -> Result<Self, Error> {
        let elev = f64::from_str(s.trim()).map_err(|_| Error::InvalidElevation)?;
        if !(0.0..=90.0).contains(&elev) {
            Err(Error::InvalidElevation)
        } else {
            Ok(Self::Elevation(elev))
        }
    }
    pub fn parse_azimuth(s: &str) -> Result<Self, Error> {
        let azim = f64::from_str(s.trim()).map_err(|_| Error::InvalidAzimuth)?;
        if !(0.0..=360.0).contains(&azim) {
            Err(Error::InvalidAzimuth)
        } else {
            Ok(Self::Azimuth(azim))
        }
    }
    pub fn parse_frequencies(s: &str) -> Result<Self, Error> {
        let freqz = s
            .split(',')
            .filter_map(|s| {
                if let Ok(f) = f64::from_str(s.trim()) {
                    Some(f)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if freqz.is_empty() {
            Err(Error::InvalidFrequency)
        } else {
            Ok(Self::Frequencies(freqz))
        }
    }
    pub fn parse_constellations(s: &str) -> Result<Self, Error> {
        let constells = s
            .trim()
            .split(',')
            .filter_map(|s| {
                if let Ok(c) = Constellation::from_str(s.trim()) {
                    Some(c)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if constells.is_empty() {
            Err(Error::InvalidConstellation)
        } else {
            Ok(Self::Constellations(constells))
        }
    }
}
