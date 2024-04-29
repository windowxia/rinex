use bitflags::bitflags;
use itertools::Itertools;
use std::collections::{
    btree_map::{Iter, IterMut},
    BTreeMap, HashMap,
};
use std::str::FromStr;
use thiserror::Error;

use crate::{
    epoch::{
        format as epoch_formatter, parse_in_timescale as parse_epoch_in_timescale,
        parse_utc as parse_utc_epoch, ParsingError as EpochParsingError,
    },
    merge::{Error as MergeError, Merge},
    observation::{flag::Error as FlagParsingError, EpochFlag, SNR},
    prelude::{Constellation, Duration, Epoch, Header, RinexType, TimeScale, SV},
    split::{Error as SplitError, Split},
    version::Version,
    Observable,
};

use gnss::{
    constellation::ParsingError as ConstellationParsingError, sv::ParsingError as SvParsingError,
};

#[cfg(feature = "qc")]
use rinex_qc_traits::{MaskFilter, MaskOperand, MaskToken, Masking};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse epoch flag")]
    EpochFlag(#[from] FlagParsingError),
    #[error("failed to parse epoch")]
    EpochError(#[from] EpochParsingError),
    #[error("sv parsing error")]
    SvParsing(#[from] SvParsingError),
    #[error("constellation parsing error")]
    ConstellationParsing(#[from] ConstellationParsingError),
    #[error("line is empty")]
    EmptyLine,
    #[error("failed to parser number of SV")]
    NumSatParsing,
    #[error("missing SV description")]
    MissingSvDescription,
    #[error("missing OBS description")]
    MissingObservationDescription,
    #[error("invalid constellation definition")]
    InvalidConstellationDefinition,
}

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    #[derive(Debug, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct LliFlags: u8 {
        /// Current epoch is marked Ok or Unknown status
        const OK_OR_UNKNOWN = 0x00;
        /// Lock lost between previous observation and current observation,
        /// cycle slip is possible
        const LOCK_LOSS = 0x01;
        /// Half cycle slip marker
        const HALF_CYCLE_SLIP = 0x02;
        /// Observing under anti spoofing,
        /// might suffer from decreased SNR - decreased signal quality
        const UNDER_ANTI_SPOOFING = 0x04;
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ObservationData {
    /// Actual observation.
    /// Unit and meaning depends [Observable] used as index Key.
    pub value: f64,
    /// Loss of lock indication, supports bitmasking.
    pub lli: Option<LliFlags>,
    /// Possible SNR information
    pub snr: Option<SNR>,
}

impl std::ops::Add for ObservationData {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            lli: self.lli,
            snr: self.snr,
            value: self.value + rhs.value,
        }
    }
}

impl std::ops::AddAssign for ObservationData {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
    }
}

impl ObservationData {
    /// Builds new [Self]
    pub fn new(value: f64, lli: Option<LliFlags>, snr: Option<SNR>) -> ObservationData {
        ObservationData { value, lli, snr }
    }
    /// Self is declared `ok` if no perturbations event are declared.
    /// If LLI exists:    
    ///    + LLI must match the LliFlags::OkOrUnknown flag (strictly)    
    /// if SSI exists:    
    ///    + SNR must match the .is_ok() criteria, refer to API
    pub fn is_ok(self) -> bool {
        let lli_ok = self.lli.unwrap_or(LliFlags::OK_OR_UNKNOWN) == LliFlags::OK_OR_UNKNOWN;
        let snr_ok = self.snr.unwrap_or_default().strong();
        lli_ok && snr_ok
    }

    /// Returns true if self is considered Ok with respect to given
    /// SNR condition (>=)
    pub fn is_ok_snr(&self, min_snr: SNR) -> bool {
        if self
            .lli
            .unwrap_or(LliFlags::OK_OR_UNKNOWN)
            .intersects(LliFlags::OK_OR_UNKNOWN)
        {
            if let Some(snr) = self.snr {
                snr >= min_snr
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Returns Real Distance, by converting observed pseudo range,
    /// and compensating for distant and local clock offsets.
    /// See [p17-p18 of the RINEX specifications]. It makes only
    /// sense to apply this method on Pseudo Range observations.
    /// - rcvr_offset: receiver clock offset for this epoch, given in file
    /// - sv_offset: sv clock offset
    /// - bias: other (optionnal..) additive biases
    pub fn pr_real_distance(&self, rcvr_offset: f64, sv_offset: f64, biases: f64) -> f64 {
        self.value + 299_792_458.0_f64 * (rcvr_offset - sv_offset) + biases
    }
}

/// Observation Record content
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub inner: BTreeMap<RecordKey, RecordEntry>,
}

impl Record {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }
    pub fn get(&self, key: &RecordKey) -> Option<&RecordEntry> {
        self.inner.get(key)
    }
    pub fn insert(&mut self, k: RecordKey, v: RecordEntry) {
        self.inner.insert(k, v);
    }
    pub fn get_mut(&mut self, key: &RecordKey) -> Option<&mut RecordEntry> {
        self.inner.get_mut(key)
    }
    pub fn iter(&self) -> Iter<'_, RecordKey, RecordEntry> {
        self.inner.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, RecordKey, RecordEntry> {
        self.inner.iter_mut()
    }
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&RecordKey, &mut RecordEntry) -> bool,
    {
        self.inner.retain(f)
    }
}

/// RecordKey is the Observation Record indexer
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RecordKey {
    /// Sampling [Epoch]
    pub epoch: Epoch,
    /// [EpochFlag] provides more information on sampling context
    pub flag: EpochFlag,
}

/// ObservationKey is the Observations content indexer
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ObservationKey {
    /// [SV]: Satellite Vehicle
    pub sv: SV,
    /// [Observable] determines the actual measurement
    pub observable: Observable,
}

/// Observations
pub type Observations = BTreeMap<ObservationKey, ObservationData>;

/// Record Entry
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct RecordEntry {
    /// RX Clock offset to timescale, expressed in [s]
    pub clock_offset: Option<f64>,
    /// List of [ObservationData] sorted by [ObservationKey]
    pub observations: Observations,
}

/// Returns true if given content matches a new OBSERVATION data epoch
pub(crate) fn is_new_epoch(line: &str, v: Version) -> bool {
    if v.major < 3 {
        if line.len() < 30 {
            false
        } else {
            // SPLICE flag handling (still an Observation::flag)
            let significant = !line[0..26].trim().is_empty();
            let epoch = parse_utc_epoch(&line[0..26]);
            let flag = EpochFlag::from_str(line[26..29].trim());
            if significant {
                epoch.is_ok() && flag.is_ok()
            } else if flag.is_err() {
                false
            } else {
                match flag.unwrap() {
                    EpochFlag::AntennaBeingMoved
                    | EpochFlag::NewSiteOccupation
                    | EpochFlag::HeaderInformationFollows
                    | EpochFlag::ExternalEvent => true,
                    _ => false,
                }
            }
        }
    } else {
        // Modern RINEX has a simple marker, like all V4 modern files
        match line.chars().next() {
            Some(c) => {
                c == '>' // epochs always delimited
                         // by this new identifier
            },
            _ => false,
        }
    }
}

/// Parses [RecordEntry] from given Epoch content
pub(crate) fn parse_epoch(
    header: &Header,
    content: &str,
    ts: TimeScale,
) -> Result<(RecordKey, RecordEntry), Error> {
    let mut lines = content.lines();
    let mut line = match lines.next() {
        Some(l) => l,
        _ => return Err(Error::EmptyLine),
    };

    // Epoch
    let (epoch, rem) = match header.version.major {
        1 | 2 => {
            let offset = " 21 12 21  0  0 30.0000000".len();
            let (datetime, rem) = line.split_at(offset);
            let epoch = parse_epoch_in_timescale(datetime, ts)?;
            (epoch, rem)
        },
        _ => {
            let line = line.split_at(1).1; // '>'
            let offset = " 2022 01 09 00 00  0.0000000".len();
            let (datetime, rem) = line.split_at(offset);
            let epoch = parse_epoch_in_timescale(datetime, ts)?;
            (epoch, rem)
        },
    };

    let (flag, rem) = rem.split_at(3);
    let flag = EpochFlag::from_str(flag.trim())?;
    let (n_sat, rem) = rem.split_at(3);
    let n_sat = n_sat
        .trim()
        .parse::<u16>()
        .map_err(|_| Error::NumSatParsing)?;

    // grab possible clock offset
    let ck_off: Option<&str> = match header.version.major < 2 {
        true => {
            // RINEX 2
            // clock offsets are last 12 characters
            if line.len() > 60 - 12 {
                Some(line.split_at(60 - 12).1.trim())
            } else {
                None
            }
        },
        false => {
            // RINEX 3
            let min_len: usize = 4+1 // y
                +2+1 // m
                +2+1 // d
                +2+1 // h
                +2+1 // m
                +11+1// s
                +3   // flag
                +3; // n_sat
            if line.len() > min_len {
                // RINEX3: clock offset precision was increased
                Some(line.split_at(min_len).1.trim()) // this handles it naturally
            } else {
                None
            }
        },
    };
    let clock_offset: Option<f64> = match ck_off {
        Some(content) => {
            if let Ok(f) = f64::from_str(content.trim()) {
                Some(f)
            } else {
                None // parsing failed for some reason
            }
        },
        None => None, // empty field
    };

    let observations = match flag {
        EpochFlag::Ok | EpochFlag::PowerFailure | EpochFlag::CycleSlip => {
            parse_observations(header, n_sat, rem, lines)?
        },
        _ => {
            /*
             * EPOCH event not handled yet
             * Best solution is most likely to return a meaningful error
             * and catch it at higher level
             */
            Default::default()
        },
    };

    Ok((
        RecordKey { epoch, flag },
        RecordEntry {
            clock_offset,
            observations,
        },
    ))
}

fn parse_observations(
    header: &Header,
    n_sat: u16,
    rem: &str,
    mut lines: std::str::Lines<'_>,
) -> Result<Observations, Error> {
    // retrive header section
    let obs = header.obs.as_ref().unwrap();
    let observables = &obs.codes;
    match header.version.major {
        2 => {
            // SV system descriptor
            let mut systems = String::with_capacity(24 * 3); //SVNN
            systems.push_str(rem.trim()); // first line always contained
                                          // append the required amount of extra lines
            let extra = if n_sat < 13 { 0 } else { n_sat / 13 };
            for _ in 0..extra {
                if let Some(l) = lines.next() {
                    systems.push_str(l.trim());
                } else {
                    return Err(Error::MissingSvDescription);
                }
            }
            Ok(parse_v2_observations(header, &systems, observables, lines)?)
        },
        _ => Ok(parse_v3_observations(observables, lines)?),
    }
}

/*
 * Parses a V2 epoch from given lines iteratoor
 * Vehicle description is contained in the epoch descriptor
 * Each vehicle content is wrapped into several lines
 */
fn parse_v2_observations(
    header: &Header,
    systems: &str,
    header_observables: &HashMap<Constellation, Vec<Observable>>,
    lines: std::str::Lines<'_>,
) -> Result<Observations, Error> {
    let svnn_size = 3; // SVNN standard
    let nb_max_observables = 5; // in a single line
    let observable_width = 16; // data + 2 flags + 1 whitespace
    let mut sv_ptr = 0; // svnn pointer
    let mut obs_ptr = 0; // observable pointer
    let mut sv = SV::default();
    let mut observables: &Vec<Observable>;
    let mut observations = Observations::new();
    // dbg!("\"{}\"", systems);

    // parse first system we're dealing with
    if systems.len() < svnn_size {
        // can't even parse a single vehicle;
        return Err(Error::EmptyLine);
    }

    /*
     * identify 1st system
     */
    let max = std::cmp::min(svnn_size, systems.len()); // for epochs with a single vehicle
    let system = &systems[0..max];

    if let Ok(ssv) = SV::from_str(system) {
        sv = ssv;
    } else {
        // may fail on omitted X in "XYY",
        // OLD RINEX.........
        match header.constellation {
            Some(Constellation::Mixed) | None => {
                return Err(Error::InvalidConstellationDefinition);
            },
            Some(c) => {
                if let Ok(prn) = system.trim().parse::<u8>() {
                    sv = SV::from_str(&format!("{:x}{:02}", c, prn))?;
                }
            },
        }
    }

    // dbg!("\"{}\"={}", system, sv);
    sv_ptr += svnn_size; // increment pointer

    // grab observables for this vehicle
    observables = match sv.constellation.is_sbas() {
        true => {
            if let Some(observables) = header_observables.get(&Constellation::SBAS) {
                observables
            } else {
                return Err(Error::MissingObservationDescription);
            }
        },
        false => {
            if let Some(observables) = header_observables.get(&sv.constellation) {
                observables
            } else {
                return Err(Error::MissingObservationDescription);
            }
        },
    };
    //println!("{:?}", observables); // DEBUG

    for line in lines {
        // browse all lines provided
        //println!("parse_v2: \"{}\"", line); //DEBUG
        let line_width = line.len();
        if line_width < 10 {
            // line is empty: add maximal amount of vehicles possible
            //println!("\nEMPTY LINE: \"{}\"", line); //DEBUG
            obs_ptr += std::cmp::min(nb_max_observables, observables.len() - obs_ptr);
        } else {
            //println!("\nLINE: \"{}\"", line); //DEBUG
            let nb_obs = num_integer::div_ceil(line_width, observable_width); // nb observations in this line

            // println!("NB OBS: {}", nb_obs); //DEBUG
            for i in 0..nb_obs {
                obs_ptr += 1;
                if obs_ptr > observables.len() {
                    // line is abnormally long compared to header definitions
                    //  parsing would fail
                    break;
                }
                let slice = match i {
                    0 => &line[0..std::cmp::min(17, line_width)],
                    _ => {
                        let start = i * observable_width;
                        let end = std::cmp::min((i + 1) * observable_width, line_width);
                        &line[start..end]
                    },
                };
                let obs = &slice[0..std::cmp::min(slice.len(), 14)]; // trimmed observations
                let mut lli: Option<LliFlags> = None;
                let mut snr: Option<SNR> = None;
                if let Ok(value) = obs.trim().parse::<f64>() {
                    // parse obs
                    if slice.len() > 13 {
                        let lli_str = &slice[13..14];
                        if let Ok(u) = lli_str.parse::<u8>() {
                            lli = LliFlags::from_bits(u);
                        }
                        if slice.len() > 14 {
                            let snr_str = &slice[14..15];
                            if let Ok(s) = SNR::from_str(snr_str) {
                                snr = Some(s);
                            }
                        }
                    }
                    //println!(
                    //    "{} {:?} ==> {}|{:?}|{:?}",
                    //    sv,
                    //    observables[obs_ptr - 1],
                    //    value,
                    //    lli,
                    //    snr
                    //); //DEBUG
                    observations.insert(
                        ObservationKey {
                            sv,
                            observable: observables[obs_ptr - 1].clone(),
                        },
                        ObservationData { value, lli, snr },
                    );
                } //f64::obs
            } // parsing all observations
            if nb_obs < nb_max_observables {
                obs_ptr += nb_max_observables - nb_obs;
            }
        }
        //println!("OBS COUNT {}", obs_ptr); //DEBUG

        if obs_ptr >= observables.len() {
            obs_ptr = 0;
            //identify next vehicle
            if sv_ptr >= systems.len() {
                // last vehicle
                return Ok(observations);
            }
            // identify next vehicle
            let start = sv_ptr;
            let end = std::cmp::min(sv_ptr + svnn_size, systems.len()); // trimed epoch description
            let system = &systems[start..end];
            if let Ok(s) = SV::from_str(system) {
                sv = s;
            } else {
                // may fail on omitted X in "XYY",
                // mainly on OLD RINEX with mono constellation
                match header.constellation {
                    Some(c) => {
                        if let Ok(prn) = system.trim().parse::<u8>() {
                            sv = SV::from_str(&format!("{:x}{:02}", c, prn))?;
                        }
                    },
                    _ => unreachable!(),
                }
            }
            // println!("\"{}\"={}", system, sv); //DEBUG
            sv_ptr += svnn_size; // increment pointer

            // grab observables for this vehicle
            observables = match sv.constellation.is_sbas() {
                true => {
                    if let Some(observables) = header_observables.get(&Constellation::SBAS) {
                        observables
                    } else {
                        return Err(Error::MissingObservationDescription);
                    }
                },
                false => {
                    if let Some(observables) = header_observables.get(&sv.constellation) {
                        observables
                    } else {
                        return Err(Error::MissingObservationDescription);
                    }
                },
            };
            //println!("{:?}", observables); // DEBUG
        }
    } // for all lines provided
    Ok(observations)
}

/*
 * Parses a V3 epoch from given lines iteratoor
 * Format is much simpler, one vehicle is described in a single line
 */
fn parse_v3_observations(
    header: &HashMap<Constellation, Vec<Observable>>,
    lines: std::str::Lines<'_>,
) -> Result<Observations, Error> {
    let svnn_size = 3; // SVNN standard
    let observable_width = 16; // data + 2 flags
    let mut observations = Observations::new();
    // browse all lines
    for line in lines {
        //println!("parse_v3: \"{}\"", line); //DEBUG
        let (sv, line) = line.split_at(svnn_size);
        let sv = SV::from_str(sv)?;
        let observables = match sv.constellation.is_sbas() {
            true => {
                if let Some(observables) = header.get(&Constellation::SBAS) {
                    observables
                } else {
                    return Err(Error::MissingObservationDescription);
                }
            },
            false => {
                if let Some(observables) = header.get(&sv.constellation) {
                    observables
                } else {
                    return Err(Error::MissingObservationDescription);
                }
            },
        };
        //println!("SV: {} OBSERVABLES: {:?}", sv, observables); // DEBUG
        let nb_obs = line.len() / observable_width;
        let mut rem = line;
        for i in 0..nb_obs {
            if i == observables.len() {
                // content does not match header description
                break;
            }
            let split_offset = std::cmp::min(observable_width, rem.len()); // avoid overflow on last obs
            let (content, r) = rem.split_at(split_offset);
            //println!("content \"{}\" \"{}\"", content, r); //DEBUG
            rem = r;
            let content_len = content.len();
            let mut snr: Option<SNR> = None;
            let mut lli: Option<LliFlags> = None;
            let obs = &content[0..std::cmp::min(observable_width - 2, content_len)];
            //println!("OBS \"{}\"", obs); //DEBUG
            if let Ok(value) = f64::from_str(obs.trim()) {
                if content_len > observable_width - 2 {
                    let lli_str = &content[observable_width - 2..observable_width - 1];
                    if let Ok(u) = u8::from_str_radix(lli_str, 10) {
                        lli = LliFlags::from_bits(u);
                    }
                }
                if content_len > observable_width - 1 {
                    let snr_str = &content[observable_width - 1..observable_width];
                    if let Ok(s) = SNR::from_str(snr_str) {
                        snr = Some(s);
                    }
                }
                //println!("LLI {:?}", lli); //DEBUG
                //println!("SSI {:?}", snr);
                // build content
                observations.insert(
                    ObservationKey {
                        sv,
                        observable: observables[i].clone(),
                    },
                    ObservationData { value, lli, snr },
                );
            }
        }
        if rem.len() >= observable_width - 2 {
            let mut snr: Option<SNR> = None;
            let mut lli: Option<LliFlags> = None;
            let obs = &rem[0..observable_width - 2];
            if let Ok(value) = obs.trim().parse::<f64>() {
                if rem.len() > observable_width - 2 {
                    let lli_str = &rem[observable_width - 2..observable_width - 1];
                    if let Ok(u) = lli_str.parse::<u8>() {
                        lli = LliFlags::from_bits(u);
                        if rem.len() > observable_width - 1 {
                            let snr_str = &rem[observable_width - 1..];
                            if let Ok(s) = SNR::from_str(snr_str) {
                                snr = Some(s);
                            }
                        }
                    }
                }
                observations.insert(
                    ObservationKey {
                        sv,
                        observable: observables[nb_obs].clone(),
                    },
                    ObservationData { value, lli, snr },
                );
            }
        }
    } //browse all lines
    Ok(observations)
}

/// Formats one epoch according to standard definitions
pub(crate) fn fmt_epoch(
    header: &Header,
    key: &RecordKey,
    entry: &RecordEntry,
) -> Result<String, Error> {
    if header.version.major < 3 {
        fmt_epoch_v2(header, key, entry)
    } else {
        fmt_epoch_v3(header, key, entry)
    }
}

fn fmt_epoch_v3(header: &Header, key: &RecordKey, entry: &RecordEntry) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);

    let (epoch, flag) = (key.epoch, key.flag);
    let clock_offset = entry.clock_offset;
    let observations = &entry.observations;
    let observables = &header.obs.as_ref().unwrap().codes;

    let unique_sv = observations
        .iter()
        .map(|(k, _)| k.sv)
        .sorted()
        .unique()
        .collect::<Vec<_>>();

    lines.push_str(&format!(
        "> {}  {} {:2}",
        epoch_formatter(epoch, RinexType::ObservationData, 3),
        flag,
        unique_sv.len(),
    ));

    if let Some(data) = clock_offset {
        lines.push_str(&format!("{:13.4}", data));
    }
    lines.push('\n');

    for sv in unique_sv {
        lines.push_str(&format!("{:x}", sv));

        // determine observation table
        let observables = if sv.constellation.is_sbas() {
            observables.get(&Constellation::SBAS)
        } else {
            observables.get(&sv.constellation)
        };

        let observables = match observables {
            Some(observables) => observables,
            None => {
                return Err(Error::MissingObservationDescription);
            },
        };

        for observable in observables {
            if let Some(obs_data) = observations.get(&ObservationKey {
                sv,
                observable: observable.clone(),
            }) {
                lines.push_str(&format!("{:14.3}", obs_data.value));

                if let Some(flag) = obs_data.lli {
                    lines.push_str(&format!("{}", flag.bits()));
                } else {
                    lines.push(' ');
                }

                if let Some(flag) = obs_data.snr {
                    lines.push_str(&format!("{:x}", flag));
                } else {
                    lines.push(' ');
                }
            } else {
                // missing observations are blanked
                lines.push_str("                ");
            }
        }
        lines.push('\n');
    }
    lines.truncate(lines.trim_end().len());
    Ok(lines)
}

fn fmt_epoch_v2(header: &Header, key: &RecordKey, entry: &RecordEntry) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);

    let (epoch, flag) = (key.epoch, key.flag);
    let clock_offset = entry.clock_offset;
    let observations = &entry.observations;
    let observables = &header.obs.as_ref().unwrap().codes;

    let unique_sv = observations
        .iter()
        .map(|(k, _)| k.sv)
        .sorted()
        .unique()
        .collect::<Vec<_>>();

    // format first line: starts with Epoch
    lines.push_str(&format!(
        " {}  {} {:2}",
        epoch_formatter(epoch, RinexType::ObservationData, 2),
        flag,
        unique_sv.len(),
    ));

    for (index, sv) in unique_sv.iter().enumerate() {
        if index == 12 {
            // clock offset on first time
            if let Some(clk) = clock_offset {
                lines.push_str(&format!(" {:9.1}", clk));
            }
        }
        if (index + 1) % 13 == 0 {
            // tab
            lines.push_str("\n                                ");
        }
        lines.push_str(&format!("{:x}", sv));
    }

    for sv in unique_sv {
        // retrieve observables table
        let observables = if sv.constellation.is_sbas() {
            observables.get(&Constellation::SBAS)
        } else {
            observables.get(&sv.constellation)
        };

        let observables = match observables {
            Some(observables) => observables,
            None => {
                return Err(Error::MissingObservationDescription);
            },
        };

        for (obs_index, observable) in observables.iter().sorted().enumerate() {
            if obs_index % 5 == 0 {
                lines.push('\n');
            }
            if let Some(obs_data) = observations.get(&ObservationKey {
                sv,
                observable: observable.clone(),
            }) {
                if obs_index % 5 == 0 {
                    lines.push_str(&format!("{:12.3}", obs_data.value));
                } else {
                    lines.push_str(&format!("{:14.3}", obs_data.value));
                }
                if let Some(flag) = obs_data.lli {
                    lines.push_str(&format!("{:x}", flag.bits()));
                } else {
                    lines.push_str(" ");
                }
                if let Some(snr) = obs_data.snr {
                    lines.push_str(&format!("{:x}", snr));
                } else {
                    lines.push_str(" ");
                }
            } else {
                // missing observations are blanked
                lines.push_str("                ");
            }
        }
    }
    lines.truncate(lines.trim_end().len());
    Ok(lines)
}

impl Merge for Record {
    /// Merge `rhs` into `Self`
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merge `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        for (rhs_k, rhs_v) in rhs.iter() {
            if let Some(lhs_v) = self.get_mut(&rhs_k) {
                for (rhs_k, rhs_v) in rhs_v.observations.iter() {}
            } else {
                // Declare new Epoch
                self.insert(rhs_k.clone(), rhs_v.clone());
            }
        }
        Ok(())
    }
}

impl Split for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), SplitError> {
        let r0 = self
            .iter()
            .flat_map(|(k, v)| {
                if k.epoch < epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k.epoch >= epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((Self { inner: r0 }, Self { inner: r1 }))
    }
    fn split_dt(&self, duration: Duration) -> Result<Vec<Self>, SplitError> {
        let mut curr = Self::new();
        let mut ret: Vec<Self> = Vec::new();
        let mut prev: Option<Epoch> = None;
        for (k, v) in self.iter() {
            if let Some(p_epoch) = prev {
                let dt = k.epoch - p_epoch;
                if dt >= duration {
                    prev = Some(k.epoch);
                    ret.push(curr);
                    curr = Self::new();
                }
                curr.insert(k.clone(), v.clone());
            } else {
                prev = Some(k.epoch);
            }
        }
        Ok(ret)
    }
}

//TODO: Hatch smoothing filter
//    fn hatch_smoothing_mut(&mut self) {
//        // buffer:
//        // stores n index, previously associated phase point and previous result
//        // for every observable we're computing
//        let mut buffer: HashMap<SV, HashMap<Observable, (f64, f64, f64)>> = HashMap::new();
//        // for each pseudo range observation for all epochs,
//        // the operation is only feasible if an associated phase_point exists
//        //   Ex: C1C with L1C, not L1W
//        //   and C2P with L2P not L2W
//        for (_, (_, svs)) in self.iter_mut() {
//            for (sv, observables) in svs.iter_mut() {
//                let rhs_observables = observables.clone();
//                for (pr_observable, pr_observation) in observables.iter_mut() {
//                    if !pr_observable.is_pseudorange_observable() {
//                        continue;
//                    }
//
//                    let pr_code = pr_observable.code().unwrap();
//
//                    // locate associated L code
//                    let ph_tolocate = "L".to_owned() + &pr_code;
//
//                    let mut ph_data: Option<f64> = None;
//                    for (rhs_observable, rhs_observation) in &rhs_observables {
//                        let rhs_code = rhs_observable.to_string();
//                        if rhs_code == ph_tolocate {
//                            ph_data = Some(rhs_observation.obs);
//                            break;
//                        }
//                    }
//
//                    if ph_data.is_none() {
//                        continue; // can't progress at this point
//                    }
//
//                    let phase_data = ph_data.unwrap();
//
//                    if let Some(data) = buffer.get_mut(sv) {
//                        if let Some((n, prev_result, prev_phase)) = data.get_mut(pr_observable) {
//                            let delta_phase = phase_data - *prev_phase;
//                            // implement corrector equation
//                            pr_observation.obs = 1.0 / *n * pr_observation.obs
//                                + (*n - 1.0) / *n * (*prev_result + delta_phase);
//                            // update buffer storage for next iteration
//                            *n += 1.0_f64;
//                            *prev_result = pr_observation.obs;
//                            *prev_phase = phase_data;
//                        } else {
//                            // first time we encounter this observable
//                            // initiate buffer
//                            data.insert(
//                                pr_observable.clone(),
//                                (2.0_f64, pr_observation.obs, phase_data),
//                            );
//                        }
//                    } else {
//                        // first time we encounter this sv
//                        // pr observation is untouched on S(0)
//                        // initiate buffer
//                        let mut map: HashMap<Observable, (f64, f64, f64)> = HashMap::new();
//                        map.insert(
//                            pr_observable.clone(),
//                            (2.0_f64, pr_observation.obs, phase_data),
//                        );
//                        buffer.insert(*sv, map);
//                    }
//                }
//            }
//        }
//    }

#[cfg(feature = "qc")]
#[cfg_attr(docrs, doc(cfg(feature = "qc")))]
impl Masking for Record {
    fn mask(&self, mask: &MaskFilter) -> Self {
        let mut s = self.clone();
        s.mask_mut(mask);
        s
    }
    fn mask_mut(&mut self, mask: &MaskFilter) {
        match &mask.token {
            MaskToken::Epoch(t) => {
                self.retain(|k, _| match mask.operand {
                    MaskOperand::Equals => k.epoch == *t,
                    MaskOperand::NotEquals => k.epoch != *t,
                    MaskOperand::GreaterThan => k.epoch > *t,
                    MaskOperand::GreaterEquals => k.epoch >= *t,
                    MaskOperand::LowerThan => k.epoch < *t,
                    MaskOperand::LowerEquals => k.epoch <= *t,
                });
            },
            MaskToken::Duration(dt) => {},
            MaskToken::Frequencies(freqz) => {},
            MaskToken::SNR(snr) => {
                self.retain(|_, v| {
                    v.observations.retain(|_, v| {
                        if let Some(snr_i) = v.snr {
                            let snr_db: f64 = snr_i.into();
                            match mask.operand {
                                MaskOperand::Equals => snr_db == *snr,
                                MaskOperand::NotEquals => snr_db != *snr,
                                MaskOperand::GreaterThan => snr_db > *snr,
                                MaskOperand::GreaterEquals => snr_db >= *snr,
                                MaskOperand::LowerThan => snr_db < *snr,
                                MaskOperand::LowerEquals => snr_db <= *snr,
                            }
                        } else {
                            false
                        }
                    });
                    !v.observations.is_empty() || v.clock_offset.is_some()
                });
            },
            MaskToken::SV(svs) => {
                self.retain(|_, v| {
                    v.observations.retain(|k, _| match mask.operand {
                        MaskOperand::Equals => svs.contains(&k.sv),
                        MaskOperand::NotEquals => !svs.contains(&k.sv),
                        MaskOperand::GreaterThan => {
                            if let Some(min_prn) = svs
                                .iter()
                                .filter_map(|sv_i| {
                                    if sv_i.constellation == k.sv.constellation {
                                        Some(sv_i.prn)
                                    } else {
                                        None
                                    }
                                })
                                .reduce(|k, _| k)
                            {
                                k.sv.prn > min_prn
                            } else {
                                true
                            }
                        },
                        MaskOperand::GreaterEquals => {
                            if let Some(min_prn) = svs
                                .iter()
                                .filter_map(|sv_i| {
                                    if sv_i.constellation == k.sv.constellation {
                                        Some(sv_i.prn)
                                    } else {
                                        None
                                    }
                                })
                                .reduce(|k, _| k)
                            {
                                k.sv.prn >= min_prn
                            } else {
                                true
                            }
                        },
                        MaskOperand::LowerThan => {
                            if let Some(max_prn) = svs
                                .iter()
                                .filter_map(|sv_i| {
                                    if sv_i.constellation == k.sv.constellation {
                                        Some(sv_i.prn)
                                    } else {
                                        None
                                    }
                                })
                                .reduce(|k, _| k)
                            {
                                k.sv.prn < max_prn
                            } else {
                                true
                            }
                        },
                        MaskOperand::LowerEquals => {
                            if let Some(max_prn) = svs
                                .iter()
                                .filter_map(|sv_i| {
                                    if sv_i.constellation == k.sv.constellation {
                                        Some(sv_i.prn)
                                    } else {
                                        None
                                    }
                                })
                                .reduce(|k, _| k)
                            {
                                k.sv.prn <= max_prn
                            } else {
                                true
                            }
                        },
                    });
                    !v.observations.is_empty() || v.clock_offset.is_some()
                });
            },
            MaskToken::Observables(obs_list) => {
                let obs_list = obs_list
                    .iter()
                    .filter_map(|obs| {
                        if let Ok(obs) = Observable::from_str(obs) {
                            Some(obs)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                self.retain(|_, v| {
                    v.observations.retain(|k, _| {
                        match mask.operand {
                            MaskOperand::Equals => obs_list.contains(&k.observable),
                            MaskOperand::NotEquals => !obs_list.contains(&k.observable),
                            _ => true, // does not apply
                        }
                    });
                    !v.observations.is_empty() || v.clock_offset.is_some()
                });
            },
            MaskToken::Constellations(constells) => {
                let mut broad_sbas_filter = false;
                for c in constells {
                    broad_sbas_filter |= *c == Constellation::SBAS;
                }
                self.retain(|_, v| {
                    v.observations.retain(|k, _| {
                        match mask.operand {
                            MaskOperand::Equals => {
                                let mut retain = constells.contains(&k.sv.constellation);
                                if broad_sbas_filter {
                                    retain |= k.sv.constellation.is_sbas();
                                }
                                retain
                            },
                            MaskOperand::NotEquals => {
                                if broad_sbas_filter {
                                    !k.sv.constellation.is_sbas()
                                        || !constells.contains(&k.sv.constellation)
                                } else {
                                    !constells.contains(&k.sv.constellation)
                                }
                            },
                            _ => true, // does not apply
                        }
                    });
                    !v.observations.is_empty() || v.clock_offset.is_some()
                });
            },
            /*
             * Following list does not apply to OBS RINEX
             */
            MaskToken::DOMES(_) => {},
            MaskToken::COSPAR(_) => {}, // TODO ?
            MaskToken::Stations(_) => {},
            MaskToken::Elevation(_) => {},
            MaskToken::Azimuth(_) => {},
        }
    }
}

// #[cfg(feature = "obs")]
// use crate::observation::{Combination, Combine};

// /*
//  * Combines same physics but observed on different carrier frequency
//  */
// #[cfg(feature = "obs")]
// fn dual_freq_combination(
//     rec: &Record,
//     combination: Combination,
// ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
//     let mut ret: HashMap<
//         (Observable, Observable),
//         BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>,
//     > = HashMap::new();
//     for (epoch, (_, vehicles)) in rec {
//         for (sv, observations) in vehicles {
//             for (lhs_observable, lhs_data) in observations {
//                 if !lhs_observable.is_phase_observable()
//                     && !lhs_observable.is_pseudorange_observable()
//                 {
//                     continue; // only for these two physics
//                 }
//
//                 // consider anything but L1
//                 let lhs_code = lhs_observable.to_string();
//                 let lhs_is_l1 = lhs_code.contains('1');
//                 if lhs_is_l1 {
//                     continue;
//                 }
//
//                 // find L1 reference observation
//                 let mut reference: Option<(Observable, f64)> = None;
//                 for (ref_observable, ref_data) in observations {
//                     let mut shared_physics = ref_observable.is_phase_observable()
//                         && lhs_observable.is_phase_observable();
//                     shared_physics |= ref_observable.is_pseudorange_observable()
//                         && lhs_observable.is_pseudorange_observable();
//                     if !shared_physics {
//                         continue;
//                     }
//
//                     let refcode = ref_observable.to_string();
//                     if refcode.contains('1') {
//                         reference = Some((ref_observable.clone(), ref_data.obs));
//                         break; // DONE searching
//                     }
//                 }
//
//                 if reference.is_none() {
//                     continue; // can't proceed further
//                 }
//                 let (ref_observable, ref_data) = reference.unwrap();
//
//                 // determine frequencies
//                 let lhs_carrier = Carrier::from_observable(sv.constellation, lhs_observable);
//                 let ref_carrier = Carrier::from_observable(sv.constellation, &ref_observable);
//                 if lhs_carrier.is_err() | ref_carrier.is_err() {
//                     continue; // undetermined frequency
//                 }
//
//                 let (lhs_carrier, ref_carrier) = (lhs_carrier.unwrap(), ref_carrier.unwrap());
//                 let (fj, fi) = (lhs_carrier.frequency(), ref_carrier.frequency());
//                 let (lambda_j, lambda_i) = (lhs_carrier.wavelength(), ref_carrier.wavelength());
//
//                 let alpha = match combination {
//                     Combination::GeometryFree => 1.0_f64,
//                     Combination::IonosphereFree => 1.0 / (fi.powi(2) - fj.powi(2)),
//                     Combination::WideLane => 1.0 / (fi - fj),
//                     Combination::NarrowLane => 1.0 / (fi + fj),
//                     Combination::MelbourneWubbena => unreachable!("mw combination"),
//                 };
//
//                 let beta = match combination {
//                     Combination::GeometryFree => 1.0_f64,
//                     Combination::IonosphereFree => fi.powi(2),
//                     Combination::WideLane | Combination::NarrowLane => fi,
//                     Combination::MelbourneWubbena => unreachable!("mw combination"),
//                 };
//
//                 let gamma = match combination {
//                     Combination::GeometryFree => 1.0_f64,
//                     Combination::IonosphereFree => fj.powi(2),
//                     Combination::WideLane | Combination::NarrowLane => fj,
//                     Combination::MelbourneWubbena => unreachable!("mw combination"),
//                 };
//
//                 let (v_j, v_i) = match combination {
//                     Combination::GeometryFree => {
//                         if ref_observable.is_pseudorange_observable() {
//                             (ref_data, lhs_data.obs)
//                         } else {
//                             (lhs_data.obs * lambda_j, ref_data * lambda_i)
//                         }
//                     },
//                     _ => {
//                         if ref_observable.is_pseudorange_observable() {
//                             (lhs_data.obs, ref_data)
//                         } else {
//                             (lhs_data.obs * lambda_j, ref_data * lambda_i)
//                         }
//                     },
//                 };
//
//                 let value = match combination {
//                     Combination::NarrowLane => alpha * (beta * v_i + gamma * v_j),
//                     _ => alpha * (beta * v_i - gamma * v_j),
//                 };
//
//                 let combination = (lhs_observable.clone(), ref_observable.clone());
//                 if let Some(data) = ret.get_mut(&combination) {
//                     if let Some(data) = data.get_mut(sv) {
//                         data.insert(*epoch, value);
//                     } else {
//                         let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
//                         map.insert(*epoch, value);
//                         data.insert(*sv, map);
//                     }
//                 } else {
//                     let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
//                     map.insert(*epoch, value);
//                     let mut bmap: BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>> = BTreeMap::new();
//                     bmap.insert(*sv, map);
//                     ret.insert(combination, bmap);
//                 }
//             }
//         }
//     }
//     ret
// }

// #[cfg(feature = "obs")]
// fn mw_combination(
//     rec: &Record,
// ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
//     let code_narrow = dual_freq_combination(rec, Combination::NarrowLane);
//     let mut phase_wide = dual_freq_combination(rec, Combination::WideLane);
//
//     phase_wide.retain(|(lhs_obs, rhs_obs), phase_wide| {
//         let lhs_code_obs =
//             Observable::from_str(&format!("C{}", &lhs_obs.to_string()[1..])).unwrap();
//         let rhs_code_obs =
//             Observable::from_str(&format!("C{}", &rhs_obs.to_string()[1..])).unwrap();
//
//         if lhs_obs.is_phase_observable() {
//             if let Some(code_data) = code_narrow.get(&(lhs_code_obs, rhs_code_obs)) {
//                 phase_wide.retain(|sv, phase_data| {
//                     if let Some(code_data) = code_data.get(sv) {
//                         phase_data.retain(|epoch, _| code_data.get(epoch).is_some());
//                         !phase_data.is_empty()
//                     } else {
//                         false
//                     }
//                 });
//                 !phase_wide.is_empty()
//             } else {
//                 false
//             }
//         } else {
//             false
//         }
//     });
//
//     for ((lhs_obs, rhs_obs), phase_data) in phase_wide.iter_mut() {
//         let lhs_code_obs =
//             Observable::from_str(&format!("C{}", &lhs_obs.to_string()[1..])).unwrap();
//         let rhs_code_obs =
//             Observable::from_str(&format!("C{}", &rhs_obs.to_string()[1..])).unwrap();
//
//         if let Some(code_data) = code_narrow.get(&(lhs_code_obs, rhs_code_obs)) {
//             for (phase_sv, data) in phase_data {
//                 if let Some(code_data) = code_data.get(phase_sv) {
//                     for (epoch, phase_wide) in data {
//                         if let Some(narrow_code) = code_data.get(epoch) {
//                             *phase_wide -= narrow_code;
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     phase_wide
// }

// #[cfg(feature = "obs")]
// impl Combine for Record {
//     fn combine(
//         &self,
//         c: Combination,
//     ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
//         match c {
//             Combination::GeometryFree
//             | Combination::IonosphereFree
//             | Combination::NarrowLane
//             | Combination::WideLane => dual_freq_combination(self, c),
//             Combination::MelbourneWubbena => mw_combination(self),
//         }
//     }
// }

// #[cfg(feature = "obs")]
// use crate::{
//     carrier,
//     observation::Dcb, //Mp},
// };

// #[cfg(feature = "obs")]
// impl Dcb for Record {
//     fn dcb(&self) -> HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
//         let mut ret: HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> =
//             HashMap::new();
//         for (epoch, (_, vehicles)) in self {
//             for (sv, observations) in vehicles {
//                 for (lhs_observable, lhs_observation) in observations {
//                     if !lhs_observable.is_phase_observable()
//                         && !lhs_observable.is_pseudorange_observable()
//                     {
//                         continue;
//                     }
//                     let lhs_code = lhs_observable.to_string();
//                     let lhs_carrier = &lhs_code[1..2];
//                     let lhs_code = &lhs_code[1..];
//
//                     for rhs_code in carrier::KNOWN_CODES.iter() {
//                         // locate a reference code
//                         if *rhs_code != lhs_code {
//                             // code differs
//                             if rhs_code.starts_with(lhs_carrier) {
//                                 // same carrier
//                                 let tolocate = match lhs_observable.is_phase_observable() {
//                                     true => "L".to_owned() + rhs_code,  // same physics
//                                     false => "C".to_owned() + rhs_code, // same physics
//                                 };
//                                 let tolocate = Observable::from_str(&tolocate).unwrap();
//                                 if let Some(rhs_observation) = observations.get(&tolocate) {
//                                     // got a reference code
//                                     let mut already_diffd = false;
//
//                                     for (op, vehicles) in ret.iter_mut() {
//                                         if op.contains(lhs_code) {
//                                             already_diffd = true;
//
//                                             // determine this code's role in the diff op
//                                             // so it remains consistent
//                                             let items: Vec<&str> = op.split('-').collect();
//
//                                             if lhs_code == items[0] {
//                                                 // code is differenced
//                                                 if let Some(data) = vehicles.get_mut(sv) {
//                                                     data.insert(
//                                                         *epoch,
//                                                         lhs_observation.obs - rhs_observation.obs,
//                                                     );
//                                                 } else {
//                                                     let mut bmap: BTreeMap<
//                                                         (Epoch, EpochFlag),
//                                                         f64,
//                                                     > = BTreeMap::new();
//                                                     bmap.insert(
//                                                         *epoch,
//                                                         lhs_observation.obs - rhs_observation.obs,
//                                                     );
//                                                     vehicles.insert(*sv, bmap);
//                                                 }
//                                             } else {
//                                                 // code is refered to
//                                                 if let Some(data) = vehicles.get_mut(sv) {
//                                                     data.insert(
//                                                         *epoch,
//                                                         rhs_observation.obs - lhs_observation.obs,
//                                                     );
//                                                 } else {
//                                                     let mut bmap: BTreeMap<
//                                                         (Epoch, EpochFlag),
//                                                         f64,
//                                                     > = BTreeMap::new();
//                                                     bmap.insert(
//                                                         *epoch,
//                                                         rhs_observation.obs - lhs_observation.obs,
//                                                     );
//                                                     vehicles.insert(*sv, bmap);
//                                                 }
//                                             }
//                                         }
//                                     }
//                                     if !already_diffd {
//                                         let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
//                                             BTreeMap::new();
//                                         bmap.insert(
//                                             *epoch,
//                                             lhs_observation.obs - rhs_observation.obs,
//                                         );
//                                         let mut map: BTreeMap<
//                                             SV,
//                                             BTreeMap<(Epoch, EpochFlag), f64>,
//                                         > = BTreeMap::new();
//                                         map.insert(*sv, bmap);
//                                         ret.insert(format!("{}-{}", lhs_code, rhs_code), map);
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         ret
//     }
// }

// /*
//  * Code multipath bias
//  */
// #[cfg(feature = "obs")]
// pub(crate) fn code_multipath(
//     rec: &Record,
// ) -> HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
//     let mut ret: HashMap<Observable, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> =
//         HashMap::new();
//
//     for (epoch, (_, vehicles)) in rec {
//         for (sv, observations) in vehicles {
//             for (observable, obsdata) in observations {
//                 if !observable.is_pseudorange_observable() {
//                     continue;
//                 }
//
//                 let code = observable.to_string();
//                 let carrier = &code[1..2].to_string();
//                 let code_is_l1 = code.contains('1');
//
//                 let mut phase_i = Option::<f64>::None;
//                 let mut phase_j = Option::<f64>::None;
//                 let mut f_i = Option::<f64>::None;
//                 let mut f_j = Option::<f64>::None;
//
//                 for (rhs_observable, rhs_data) in observations {
//                     if !rhs_observable.is_phase_observable() {
//                         continue;
//                     }
//                     let rhs_code = rhs_observable.to_string();
//
//                     // identify carrier signal
//                     let rhs_carrier = Carrier::from_observable(sv.constellation, rhs_observable);
//                     if rhs_carrier.is_err() {
//                         continue;
//                     }
//                     let rhs_carrier = rhs_carrier.unwrap();
//                     let lambda = rhs_carrier.wavelength();
//
//                     if code_is_l1 {
//                         if rhs_code.contains('2') {
//                             f_j = Some(rhs_carrier.frequency());
//                             phase_j = Some(rhs_data.obs * lambda);
//                         } else if rhs_code.contains(carrier) {
//                             f_i = Some(rhs_carrier.frequency());
//                             phase_i = Some(rhs_data.obs * lambda);
//                         }
//                     } else if rhs_code.contains('1') {
//                         f_j = Some(rhs_carrier.frequency());
//                         phase_j = Some(rhs_data.obs * lambda);
//                     } else if rhs_code.contains(carrier) {
//                         f_i = Some(rhs_carrier.frequency());
//                         phase_i = Some(rhs_data.obs * lambda);
//                     }
//
//                     if phase_i.is_some() && phase_j.is_some() {
//                         break; // DONE
//                     }
//                 }
//
//                 if phase_i.is_none() || phase_j.is_none() {
//                     continue; // can't proceed
//                 }
//
//                 let gamma = (f_i.unwrap() / f_j.unwrap()).powi(2);
//                 let alpha = (gamma + 1.0) / (gamma - 1.0);
//                 let beta = 2.0 / (gamma - 1.0);
//                 let value = obsdata.obs - alpha * phase_i.unwrap() + beta * phase_j.unwrap();
//
//                 if let Some(data) = ret.get_mut(observable) {
//                     if let Some(data) = data.get_mut(sv) {
//                         data.insert(*epoch, value);
//                     } else {
//                         let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
//                         map.insert(*epoch, value);
//                         data.insert(*sv, map);
//                     }
//                 } else {
//                     let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
//                     map.insert(*epoch, value);
//                     let mut bmap: BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>> = BTreeMap::new();
//                     bmap.insert(*sv, map);
//                     ret.insert(observable.clone(), bmap);
//                 }
//             }
//         }
//     }
//     ret
// }

use crate::observation::Substract;

impl Substract for Record {
    fn substract(&self, rhs: &Self) -> Self {
        let mut s = self.clone();
        s.substract_mut(rhs);
        s
    }
    fn substract_mut(&mut self, rhs: &Self) {
        self.retain(|k, v| {
            if let Some(ref_v) = rhs.get(&k) {
                v.observations.retain(|k, obs_data| {
                    if let Some(ref_data) = ref_v.observations.get(&k) {
                        obs_data.value -= ref_data.value;
                        true
                    } else {
                        false
                    }
                });
                !v.observations.is_empty()
            } else {
                false
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::{fmt_epoch, is_new_epoch, parse_epoch};
    use crate::{
        epoch::parse_utc as utc_epoch_parser,
        observation::HeaderFields,
        prelude::{Constellation, Epoch, EpochFlag, Header, Observable, TimeScale, Version},
    };
    use itertools::Itertools;
    use std::str::FromStr;
    #[test]
    fn test_parse_v2() {
        let t0 = Epoch::from_str("2021-12-21T00:00:00 GPST").unwrap();
        let mut obs_header = HeaderFields::default().with_time_of_first_obs(t0);
        for code in ["L1", "L2", "C1", "C2", "P1", "P2", "D1", "D2", "S1", "S2"] {
            let obs = Observable::from_str(code).unwrap();
            for constell in [Constellation::GPS, Constellation::Glonass] {
                if let Some(codes) = obs_header.codes.get_mut(&constell) {
                    codes.push(obs.clone());
                } else {
                    obs_header.codes.insert(constell, vec![obs.clone()]);
                }
            }
        }
        let header = Header::default()
            .with_version(Version { major: 2, minor: 0 })
            .with_constellation(Constellation::GPS)
            .with_observation_fields(obs_header);
        for (descriptor, epoch, flag, num_sat) in [
            (
                " 21 12 21  0  0  0.0000000  0  1G07
131857102.133 6 102745756.54245  25091572.300
25091565.600        -411.138        -320.373          37.350          35.300",
                Epoch::from_str("2021-12-21T00:00:00 GPST").unwrap(),
                EpochFlag::Ok,
                1,
            ),
            (
                " 21 01 01 00 00 00.0000000  0 24G07G08G10G13G15G16G18G20G21G23G26G27
                                G30R01R02R03R08R09R15R16R17R18R19R24
  24178026.635 6  24178024.891 6                 127056391.69906  99004963.01703
                  24178026.139 3  24178024.181 3        38.066          22.286  
                
  21866748.928 7  21866750.407 7  21866747.537 8 114910552.08207  89540700.32608
  85809828.27608  21866748.200 8  21866749.482 8        45.759          49.525  
        52.161  
  21458907.960 8  21458908.454 7  21458905.489 8 112767333.29708  87870655.27209
  84209365.43808  21458907.312 9  21458908.425 9        50.526          55.388  
        53.157  
  25107711.730 5                                 131941919.38305 102811868.09001
                  25107711.069 1  25107709.586 1        33.150           8.952  
                
  24224693.760 6  24224693.174 5                 127301651.00206  99196079.53805
                  24224693.407 5  24224691.898 5        36.121          31.645  
                
  21749627.212 8                                 114295057.63608  89061063.16706
                  21749626.220 6  21749624.795 6        48.078          39.240  
                
  23203962.113 6  23203960.554 6  23203963.222 7 121937655.11806  95016353.74904
  91057352.20207  23203961.787 4  23203960.356 4        41.337          28.313  
        46.834  
  21336671.709 7                                 112124979.20907  87370110.32706
                  21336670.444 6  21336669.290 6        47.463          39.510  
                
  23746180.287 6                                 124787018.18706  97236633.91403
                  23746179.022 3  23746178.067 3        38.820          22.819  
                
  21413431.070 7  21413429.404 7  21413431.981 8 112528356.08507  87684432.45406
  84030922.83008  21413430.740 6  21413429.066 6        47.698          40.362  
        52.487  
  23960478.475 6  23960480.103 6  23960477.163 7 125913155.35006  98114150.90306
  94026064.18807  23960477.733 6  23960479.641 6        39.261          36.752  
        42.698  
  20160980.296 8  20160980.485 8  20160978.441 9 105946683.25408  82555869.20609
  79116040.25909  20160979.559 9  20160980.098 9        51.584          58.520  
        55.715  
  24895095.878 6  24895095.931 5  24895094.407 6 130824617.27906 101941255.30503
  97693699.82606  24895095.087 3  24895095.779 3        37.800          20.405  
        41.373  
  21976735.287 7  21976740.713 6                 117478268.97407  91372016.95306
                                                        43.731          39.712  
                
  21452856.821 8  21452861.434 7                 114476565.58608  89037342.61407
                                                        48.976          45.633  
                
  24356366.067 6  24356369.934 6                 130381530.94906 101407869.02906
                                                        40.212          40.570  
                
  24640492.817 5  24640495.563 5                 131948754.31105 102626826.44805
                                                        31.019          35.719  
                
  22631771.515 7  22631773.097 7                 120852362.72507  93996312.56907
                                                        45.041          42.955  
                
  22333745.843 7  22333750.087 6                 119344755.47207  92823708.20506
                                                        47.198          41.178  
                
  20767116.205 7  20767118.004 7                 110934198.55007  86282150.62307
                                                        46.750          44.206  
                
  19609338.615 8  19609342.136 8                 104933562.47908  81615007.54508
                                                        53.404          49.913  
                
  20155814.670 8  20155818.459 8                 107593135.54008  83683576.27208
                                                        52.338          49.531  
                
  23769631.385 5  23769635.136 6                 127151515.87305  98895637.55806
                                                        32.323          37.626  
                
  23219147.863 6  23219153.271 6                 124163221.43806  96571415.97606
                                                        41.318          39.432",
                Epoch::from_str("2021-01-01T00:00:00 GPST").unwrap(),
                EpochFlag::Ok,
                24,
            ),
        ] {
            let parsed = parse_epoch(&header, descriptor, TimeScale::GPST);
            assert!(parsed.is_ok(), "parsing error: {}", parsed.err().unwrap());
            let (key, entry) = parsed.unwrap();
            assert_eq!(key.epoch, epoch);
            assert_eq!(key.flag, flag);
            assert!(entry.clock_offset.is_none());

            let unique_sv = entry
                .observations
                .iter()
                .map(|(k, _)| k.sv)
                .unique()
                .collect::<Vec<_>>();
            assert_eq!(unique_sv.len(), num_sat, "bad number of SV");

            let reciprocal = fmt_epoch(&header, &key, &entry);
            assert!(
                reciprocal.is_ok(),
                "failed to dump back to string: {}",
                reciprocal.err().unwrap()
            );
        }
    }
    #[test]
    fn test_parse_v3() {
        let t0 = Epoch::from_str("2021-12-21T00:00:00 GPST").unwrap();
        let mut obs_header = HeaderFields::default().with_time_of_first_obs(t0);
        for code in [
            "C1C", "L1C", "S1C", "C2S", "L2S", "S2S", "C2W", "L2W", "S2W", "C5Q", "L5Q", "S5Q",
        ] {
            let obs = Observable::from_str(code).unwrap();
            for constell in [Constellation::GPS, Constellation::Glonass] {
                if let Some(codes) = obs_header.codes.get_mut(&constell) {
                    codes.push(obs.clone());
                } else {
                    obs_header.codes.insert(constell, vec![obs.clone()]);
                }
            }
        }
        let header = Header::default()
            .with_version(Version { major: 3, minor: 0 })
            .with_constellation(Constellation::GPS)
            .with_observation_fields(obs_header);
        for (descriptor, epoch, flag, num_sat) in [
        (
"> 2022 01 09 00 00  0.0000000  0  9
G01  22345079.240   117424213.48008        48.850    22345080.640    91499404.57507        46.950    22345080.580    91499412.56807        44.500    22345078.900    87686944.70808        49.550
G03  25106377.980   131934909.06607        43.000    25106381.840   102806423.66007        42.300    25106381.060   102806435.67506        37.850    25106380.680    98522827.50406        40.650
G08  20374390.760   107068179.92108        52.300    20374392.480    83429765.07708        52.950    20374391.620    83429761.07908        52.250    20374389.640    79953524.82009        54.350
G10  22464836.260   118053596.15308        52.800    22464836.760    91989838.33208        48.900    22464836.820    91989850.33008        52.700    22464834.400    88156942.76908        49.900
R01  20207718.320   108021892.15508        49.200    20207720.980    84017055.40007        43.700    20207720.080    84017060.40107        44.650
R02  22668665.500   120964368.87508        49.350    22668667.420    94083413.28907        43.000    22668667.400    94083412.28707        43.550
R08  21849258.020   117001804.29807        45.900    21849257.240    91001400.22507        44.650    21849257.080    91001404.23007        45.300
R14  22065079.340   117619290.65008        51.050    22065080.780    91481666.44207        43.000    22065081.000    91481659.45507        43.300
R15  21933521.660   117206098.42208        51.450    21933521.820    91160302.69907        44.050    21933521.860    91160306.70907        44.650",
    Epoch::from_str("2022-01-09T00:00:00 GPST").unwrap(),
    EpochFlag::Ok,
    9,
    ),
        ] {
            let parsed = parse_epoch(&header, descriptor, TimeScale::GPST);
            assert!(parsed.is_ok(), "parsing error: {}", parsed.err().unwrap());
            let (key, entry) = parsed.unwrap();
            assert_eq!(key.epoch, epoch);
            assert_eq!(key.flag, flag);
            assert!(entry.clock_offset.is_none());

            let unique_sv = entry
                .observations
                .iter()
                .map(|(k, _)| k.sv)
                .unique()
                .collect::<Vec<_>>();
            assert_eq!(unique_sv.len(), num_sat, "bad number of SV");
        }
    }
    #[test]
    fn v2_npaz_content() {
        let t0 = Epoch::from_str("2021-12-21T00:00:00 GPST").unwrap();
        let mut obs_header = HeaderFields::default().with_time_of_first_obs(t0);
        for code in ["C1", "L1", "L2", "P2", "S1", "S2"] {
            let obs = Observable::from_str(code).unwrap();
            for constell in [Constellation::GPS, Constellation::Glonass] {
                if let Some(codes) = obs_header.codes.get_mut(&constell) {
                    codes.push(obs.clone());
                } else {
                    obs_header.codes.insert(constell, vec![obs.clone()]);
                }
            }
        }
        let header = Header::default()
            .with_version(Version { major: 2, minor: 0 })
            .with_constellation(Constellation::Mixed)
            .with_observation_fields(obs_header);
        for (descriptor, epoch, flag, num_sat) in [(
            " 21 12 21 00 47 30.0000000  0 13G01G08G10G16G21G23G32R04R05R06R12R20
                                R21
  24993483.334   131341646.26201                                        33.000  

  21063801.400   110691014.26607  86252749.55647  21063803.500          48.000  
        43.000  
  20794853.942   109277716.06007  85151471.89148  20794856.022          51.000  
        47.000  
  23287051.506   122374275.11906  95356574.87046  23287051.226          43.000  
        23.000  
  22914946.488   120418860.17306  93832876.83746  22914944.928          43.000  
        22.000  
  22514647.750   118315291.04406  92193736.44046  22514647.910          43.000  
        26.000  
  23391437.004   122922836.81105  95784029.48247  23391439.384          42.000  
        33.000  
  23309588.108   124821803.02605                                        41.000  

  20051677.568   107187721.31007  83368233.94047  20051673.008          52.000  
        34.000  
  20948630.044   111785918.59403                                        37.000  

  22904976.108   122354269.96206  95164435.36146  22904973.788          43.000  
        26.000  
  20387428.064   109020741.40904  84793909.88146  20387426.264          38.000  
        31.000  
  20504895.844   109725859.84707  85342322.66146  20504893.384          49.000  
        27.000",
            Epoch::from_str("2021-12-21T00:47:30 GPST").unwrap(),
            EpochFlag::Ok,
            13,
        )] {
            let parsed = parse_epoch(&header, descriptor, TimeScale::GPST);
            assert!(parsed.is_ok(), "parsing error: {}", parsed.err().unwrap());
            let (key, entry) = parsed.unwrap();
            assert_eq!(key.epoch, epoch);
            assert_eq!(key.flag, flag);

            let unique_sv = entry
                .observations
                .iter()
                .map(|(k, _)| k.sv)
                .unique()
                .collect::<Vec<_>>();
            assert_eq!(unique_sv.len(), num_sat, "bad number of SV");
        }
    }
    #[test]
    fn v2_aopr_content() {
        let t0 = Epoch::from_str("2017-01-01T00:00:00 GPST").unwrap();
        let mut obs_header = HeaderFields::default().with_time_of_first_obs(t0);
        for code in ["L1", "L2", "C1", "P1", "P2"] {
            let obs = Observable::from_str(code).unwrap();
            for constell in [Constellation::GPS] {
                if let Some(codes) = obs_header.codes.get_mut(&constell) {
                    codes.push(obs.clone());
                } else {
                    obs_header.codes.insert(constell, vec![obs.clone()]);
                }
            }
        }
        let header = Header::default()
            .with_version(Version { major: 2, minor: 0 })
            .with_constellation(Constellation::GPS)
            .with_observation_fields(obs_header);
        for (descriptor, epoch, flag, num_sat) in [
            (
                " 17  1  1  0  0  0.0000000  0 10G31G27G 3G32G16G 8G14G23G22G26
 -14746974.73049 -11440396.20948  22513484.6374   22513484.7724   22513487.3704
 -19651355.72649 -15259372.67949  21319698.6624   21319698.7504   21319703.7964
  -9440000.26548  -7293824.59347  23189944.5874   23189944.9994   23189951.4644
 -11141744.16748  -8631423.58147  23553953.9014   23553953.6364   23553960.7164
 -21846711.60849 -16970657.69649  20528865.5524   20528865.0214   20528868.5944
  -2919082.75648  -2211037.84947  24165234.9594   24165234.7844   24165241.6424
 -20247177.70149 -15753542.44648  21289883.9064   21289883.7434   21289887.2614
 -15110614.77049 -11762797.21948  23262395.0794   23262394.3684   23262395.3424
 -16331314.56648 -12447068.51348  22920988.2144   22920987.5494   22920990.0634
 -15834397.66049 -12290568.98049  21540206.1654   21540206.1564   21540211.9414",
                Epoch::from_str("2017-01-01T00:00:00 GPST").unwrap(),
                EpochFlag::Ok,
                10,
            ),
            (
                " 17  1  1  3 33 40.0000000  0  9G30G27G11G16G 8G 7G23G 9G 1
  -4980733.18548  -3805623.87347  24352349.1684   24352347.9244   24352356.1564
  -9710828.79748  -7513506.68548  23211317.1574   23211317.5034   23211324.2834
 -26591640.60049 -20663619.71349  20668830.8234   20668830.4204   20668833.2334
  -2876691.02148  -2188825.98947  24138743.7034   24138743.6094   24138745.3184
 -19659629.49649 -15255613.81549  20979609.7704   20979609.4094   20979615.2514
 -18951526.07649 -14757441.84348  21470398.1684   21470398.1574   21470400.8554
 -18143490.68049 -14126079.68448  22685259.0754   22685258.3664   22685261.2134
 -16594887.53049 -12883140.10148  22336785.6934   22336785.4334   22336790.8924
 -19095445.86249 -14826971.50648  21708306.6584   21708306.5704   21708312.9414",
                Epoch::from_str("2017-01-01T03:33:40 GPST").unwrap(),
                EpochFlag::Ok,
                9,
            ),
            (
                " 17  1  1  6  9 10.0000000  0 11G30G17G 3G11G19G 8G 7G 6G22G28G 1
 -23668184.66249 -18367274.15149  20796245.2334   20796244.8234   20796250.6334
  -5877878.73348  -4575160.53248  23410058.5724   23410059.2714   23410062.1064
 -14330784.79049 -11159200.76948  22386555.0924   22386555.5294   22386561.1694
 -18535782.38249 -14386326.63548  22201809.2434   22201808.6284   22201811.8674
  -2818370.49848  -2158733.26747  24199387.4244   24199386.1504   24199389.5674
  -1657187.18348  -1227738.78347  24405361.4394   24405361.8174   24405367.9104
 -20423274.04149 -15904260.09048  21190335.4504   21190335.3064   21190338.4104
  -3369328.09448  -2572763.92047  24203321.5404   24203321.3864   24203325.7804
 -14092358.97049 -10974147.19148  22566359.9814   22566358.6994   22566360.4184
 -15283523.06549 -11885593.19948  22273612.1774   22273611.9344   22273614.5104
 -21848286.72849 -16972039.81549  21184456.3894   21184456.9144   21184462.1224",
                Epoch::from_str("2017-01-01T06:09:10 GPST").unwrap(),
                EpochFlag::Ok,
                11,
            ),
        ] {
            let parsed = parse_epoch(&header, descriptor, TimeScale::GPST);
            assert!(parsed.is_ok(), "parsing error: {}", parsed.err().unwrap());
            let (key, entry) = parsed.unwrap();
            assert_eq!(key.epoch, epoch);
            assert_eq!(key.flag, flag);

            let unique_sv = entry
                .observations
                .iter()
                .map(|(k, _)| k.sv)
                .unique()
                .collect::<Vec<_>>();
            assert_eq!(unique_sv.len(), num_sat, "bad number of SV");

            let reciprocal = fmt_epoch(&header, &key, &entry);
            assert!(
                reciprocal.is_ok(),
                "failed to dump back to string: {}",
                reciprocal.err().unwrap()
            );
        }
    }
}
