use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};

use log::info;
use std::str::FromStr;

use crate::Output;
use rinex::prelude::RinexVersion;

pub struct Cli {
    /// Arguments passed by user
    matches: ArgMatches,
}

impl Cli {
    /// Build new command line interface
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("ubx2rnx")
                    .author("Guillaume W. Bres, <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("RINEX generator from UBlox device")
                    // .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(
                        Arg::new("port")
                            .short('p')
                            .long("port")
                            .value_name("PORT")
                            .help("Set device port, default: \"/dev/ttyUSB0\""),
                    )
                    .arg(
                        Arg::new("baud")
                            .short('b')
                            .long("baud")
                            .value_name("BAUDRATE")
                            .help("Set port baudrate, default: \"9600\""),
                    )
                    .arg(
                        Arg::new("version")
                            .short('v')
                            .long("ver")
                            .value_name("RINEX_VER")
                            .help("Set custom RINEX format revision, default: V4"),
                    )
                    .next_help_heading("Compression")
                    .arg(
                        Arg::new("crinex")
                            .short('c')
                            .long("crinex")
                            .action(ArgAction::SetTrue)
                            .help("Activate CRINEX compressed Observation generation"),
                    )
                    .arg(
                        Arg::new("gzip")
                            .short('g')
                            .long("gzip")
                            .action(ArgAction::SetTrue)
                            .help("Activate GZip compression"),
                    )
                    .get_matches()
            },
        }
    }
    /* returns device port to use */
    pub fn port(&self) -> String {
        if let Some(p) = self.matches.get_one::<String>("port") {
            p.clone()
        } else {
            String::from("/dev/ttyUSB0")
        }
    }
    /* returns baudrate to use */
    pub fn baudrate(&self) -> Result<u32, std::num::ParseIntError> {
        if let Some(p) = self.matches.get_one::<String>("baudrate") {
            p.parse::<u32>()
        } else {
            Ok(9600)
        }
    }
    /* returns true if Navigation Data to be generated */
    pub fn output(&self) -> Output {
        Output::StdOut
    }
    pub fn crinex(&self) -> bool {
        self.matches.get_flag("crinex")
    }
    pub fn gzip(&self) -> bool {
        self.matches.get_flag("gzip")
    }
    pub fn version(&self) -> RinexVersion {
        if let Some(custom) = self.matches.get_one::<String>("version") {
            if let Ok(vers) = RinexVersion::from_str(custom) {
                vers
            } else {
                panic!("invalid version descriptor \"{}\"", custom);
            }
        } else {
            let rev = RinexVersion::new(4, 0);
            info!("using default rinex rev: {}", rev);
            rev
        }
    }
}
