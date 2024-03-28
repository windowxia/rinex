use strum_macros::EnumString;

use crate::{
    epoch::{parse_epoch, Error as EpochParsingError},
    Error,
};

use hifitime::{Duration, Epoch};

pub mod description;
pub mod header;

#[derive(Debug, PartialEq, Clone)]
pub enum Method {
    /// Intra Frequency Bias estimation, is the analysis of differences between
    /// frequencies relying on a ionosphere reduction model.
    IntraFrequency,
    /// Inter Frequency Bias estimation, is the analysis of differences between
    /// observables of different frequencyes, relying on a ionosphere reduction model.
    InterFrequency,
    /// Results from clock analysis
    Clock,
    /// Ionosphere (geometry free) analysis
    Ionosphere,
    /// Results from Clock and Ionosphere combined analysis
    ClockIonoCombination,
}

impl std::str::FromStr for Method {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        if content.eq("CLOCK_ANALYSIS") {
            Ok(Self::Clock)
        } else if content.eq("INTRA-FREQUENCY_BIAS_ESTIMATION") {
            Ok(Self::IntraFrequency)
        } else if content.eq("INTER-FREQUENCY_BIAS_ESTIMATION") {
            Ok(Self::InterFrequency)
        } else if content.eq("IONOSPHERE_ANALYSIS") {
            Ok(Self::Ionosphere)
        } else if content.eq("COMBINED_ANALYSIS") {
            Ok(Self::ClockIonoCombination)
        } else {
            Err(Error::UnknownMethod(content.to_string()))
        }
    }
}

#[derive(Debug, PartialEq, Clone, EnumString)]
//#[derive(StrumString)]
pub enum BiasType {
    /// Differential Signal Bias (DSB)
    DSB,
    /// Ionosphere Free Signal bias (ISB)
    ISB,
    /// Observable Specific Signal bias (OSB)
    OSB,
}

#[derive(Debug, Error)]
pub enum SolutionParsingError {
    #[error("failed to parse BiasType")]
    ParseBiasTypeError(#[from] strum::ParseError),
    #[error("failed to parse bias estimate")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse epoch")]
    EpochParsingError(#[from] EpochParsingError),
}

#[derive(Debug, Clone)]
pub struct Solution {
    /// Bias type
    pub btype: BiasType,
    /// Satellite SVN
    pub svn: String,
    /// Space Vehicle ID
    pub prn: String,
    /// Station codes
    pub station: Option<String>,
    /// Observable codes used for estimating the biases,
    /// notes as (OBS1, OBS2) in standards
    pub obs: (String, Option<String>),
    /// Start time for the bias estimate
    pub start_time: Epoch,
    /// End time for the bias estimate
    pub end_time: Epoch,
    /// Bias parameter unit
    pub unit: String,
    /// Bias parameter estimate (offset)
    pub estimate: f64,
    /// Bias parameter stddev
    pub stddev: f64,
    /// Bias parameter slope estimate
    pub slope: Option<f64>,
    /// Bias parameter slope stddev estimate
    pub slope_stddev: Option<f64>,
}

impl std::str::FromStr for Solution {
    type Err = SolutionParsingError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let (bias_type, rem) = content.split_at(5);
        let (svn, rem) = rem.split_at(5);
        let (prn, rem) = rem.split_at(4);
        let (station, rem) = rem.split_at(10);
        let (obs1, rem) = rem.split_at(5);
        let (obs2, rem) = rem.split_at(5);
        let (start_time, rem) = rem.split_at(15);
        let (end_time, rem) = rem.split_at(15);
        let (unit, rem) = rem.split_at(5);
        let (estimate, rem) = rem.split_at(22);
        let (stddev, _) = rem.split_at(12);
        Ok(Solution {
            btype: BiasType::from_str(bias_type.trim())?,
            svn: svn.trim().to_string(),
            prn: prn.trim().to_string(),
            station: {
                if !station.trim().is_empty() {
                    Some(station.trim().to_string())
                } else {
                    None
                }
            },
            unit: unit.trim().to_string(),
            start_time: parse_epoch(start_time.trim())?,
            end_time: parse_epoch(end_time.trim())?,
            obs: {
                if !obs2.trim().is_empty() {
                    (obs1.trim().to_string(), Some(obs2.trim().to_string()))
                } else {
                    (obs1.trim().to_string(), None)
                }
            },
            estimate: f64::from_str(estimate.trim())?,
            stddev: f64::from_str(stddev.trim())?,
            slope: None,
            slope_stddev: None,
        })
    }
}

impl Solution {
    /// Returns duration for this bias solution
    pub fn duration(&self) -> Duration {
        self.end_time - self.start_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sinex;
    use hifitime::TimeScale;
    use std::str::FromStr;
    #[test]
    fn method_parsing() {
        for (desc, expected) in [
            ("CLOCK_ANALYSIS", Method::Clock),
            ("IONOSPHERE_ANALYSIS", Method::Ionosphere),
            ("COMBINED_ANALYSIS", Method::ClockIonoCombination),
            ("INTER-FREQUENCY_BIAS_ESTIMATION", Method::InterFrequency),
            ("INTRA-FREQUENCY_BIAS_ESTIMATION", Method::IntraFrequency),
        ] {
            let method = Method::from_str(desc).unwrap();
            assert_eq!(method, expected);
        }
    }
    #[test]
    fn solution_parser() {
        let solution = Solution::from_str(
            "ISB   G    G   GIEN      C1W  C2W  2011:113:86385 2011:115:00285 ns   0.000000000000000E+00 .000000E+00");
        assert!(solution.is_ok());
        let solution = solution.unwrap();
        assert_eq!(solution.btype, BiasType::ISB);
        assert_eq!(solution.svn, "G");
        assert_eq!(solution.prn, "G");
        assert_eq!(solution.station, Some(String::from("GIEN")));
        assert_eq!(
            solution.obs,
            (String::from("C1W"), Some(String::from("C2W")))
        );
        assert_eq!(solution.estimate, 0.0);
        assert_eq!(solution.stddev, 0.0);
        let solution = Solution::from_str(
            "ISB   E    E   GOUS      C1C  C7Q  2011:113:86385 2011:115:00285 ns   -.101593337222667E+03 .259439E+02");
        assert!(solution.is_ok());
        let solution = solution.unwrap();
        assert_eq!(solution.btype, BiasType::ISB);
        assert_eq!(solution.svn, "E");
        assert_eq!(solution.prn, "E");
        assert_eq!(solution.station, Some(String::from("GOUS")));
        assert_eq!(
            solution.obs,
            (String::from("C1C"), Some(String::from("C7Q")))
        );
        assert!((solution.estimate - -0.101593337222667E3) < 1E-6);
        assert!((solution.stddev - 0.259439E+02) < 1E-6);
        let solution = Solution::from_str(
            "OSB   G063 G01           C1C       2016:296:00000 2016:333:00000 ns                 10.2472      0.0062");
        assert!(solution.is_ok());
        let solution = solution.unwrap();
        assert_eq!(solution.btype, BiasType::OSB);
        assert_eq!(solution.svn, "G063");
        assert_eq!(solution.prn, "G01");
        assert_eq!(solution.station, None);
        assert_eq!(solution.obs, (String::from("C1C"), None));
        assert!((solution.estimate - 10.2472) < 1E-4);
        assert!((solution.stddev - 0.0062E+02) < 1E-4);
    }
    #[test]
    fn test_bia_v1_example1() {
        let file = env!("CARGO_MANIFEST_DIR").to_owned() + "/data/BIA/V1/example-1a.bia";
        let sinex = Sinex::from_file(&file);
        assert!(sinex.is_ok());
        let sinex = sinex.unwrap();
        let reference = &sinex.reference;
        assert_eq!(
            reference.description,
            "CODE, Astronomical Institute, University of Bern"
        );
        assert_eq!(
            reference.input,
            "CODE IGS 1-day final and rapid bias solutions for G/R"
        );
        assert_eq!(
            reference.output,
            "CODE IGS 30-day bias solution for G/R satellites"
        );
        assert_eq!(reference.contact, "code@aiub.unibe.ch");
        assert_eq!(reference.software, "Bernese GNSS Software Version 5.3");
        assert_eq!(reference.hardware, "UBELIX: Linux, x86_64");
        assert_eq!(sinex.acknowledgments.len(), 2);
        assert_eq!(
            sinex.acknowledgments[0],
            "COD Center for Orbit Determination in Europe, AIUB, Switzerland"
        );
        assert_eq!(sinex.acknowledgments[1], "IGS International GNSS Service");
        assert_eq!(sinex.comments.len(), 4);
        assert_eq!(sinex.comments[0], "CODE final product series for the IGS.");
        assert_eq!(
            sinex.comments[1],
            "Published by Astronomical Institute, University of Bern."
        );
        assert_eq!(
            sinex.comments[2],
            "URL: http://www.aiub.unibe.ch/download/CODE"
        );
        assert_eq!(sinex.comments[3], "DOI: 10.7892/boris.75876");

        let description = &sinex.description;
        let description = description.bias_description();
        assert!(description.is_some());
        let description = description.unwrap();
        assert_eq!(description.sampling, Some(300));
        assert_eq!(description.spacing, Some(86400));
        assert_eq!(description.method, Some(Method::ClockIonoCombination));
        assert_eq!(description.bias_mode, header::BiasMode::Absolute);
        assert_eq!(description.timescale, TimeScale::GPST);
        assert_eq!(description.rcvr_clock_ref, None);
        assert_eq!(description.sat_clock_ref.len(), 2);

        let solutions = sinex.record.bias_solutions();
        assert!(solutions.is_some());
        let solutions = solutions.unwrap();
        assert_eq!(solutions.len(), 50);
    }
    #[test]
    fn test_bia_v1_example1b() {
        let file = env!("CARGO_MANIFEST_DIR").to_owned() + "/data/BIA/V1/example-1b.bia";
        let sinex = Sinex::from_file(&file);
        assert!(sinex.is_ok());
        let sinex = sinex.unwrap();
        assert_eq!(sinex.acknowledgments.len(), 2);
        assert_eq!(
            sinex.acknowledgments[0],
            "COD Center for Orbit Determination in Europe, AIUB, Switzerland"
        );
        assert_eq!(sinex.acknowledgments[1], "IGS International GNSS Service");

        let _reference = &sinex.reference;

        let description = &sinex.description;
        let description = description.bias_description();
        assert!(description.is_some());
        let description = description.unwrap();
        assert_eq!(description.sampling, Some(300));
        assert_eq!(description.spacing, Some(86400));
        assert_eq!(description.method, Some(Method::ClockIonoCombination));
        assert_eq!(description.bias_mode, header::BiasMode::Relative);
        assert_eq!(description.timescale, TimeScale::GPST);
        assert_eq!(description.rcvr_clock_ref, None);
        assert_eq!(description.sat_clock_ref.len(), 2);

        let solutions = sinex.record.bias_solutions();
        assert!(solutions.is_some());
        let solutions = solutions.unwrap();
        assert_eq!(solutions.len(), 50);
        for sol in solutions.iter() {
            let obs = &sol.obs;
            assert!(obs.1.is_some()); // all came with OBS1+OBS2
        }
    }
}
