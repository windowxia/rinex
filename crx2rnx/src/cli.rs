use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};
use std::path::{Path, PathBuf};

pub struct Cli {
    /// arguments passed by user
    pub matches: ArgMatches,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("crx2rnx")
                    .author("Guillaume W. Bres <guillaume.bressaix@gmail.com>")
                    .version("2.0")
                    .about("Compact RINEX decompression tool")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(
                        Arg::new("input")
                            .short('i')
                            .help("Input RINEX file")
                            .required(true),
                    )
                    .arg(
                        Arg::new("workspace")
                            .short('w')
                            .long("workspace")
                            .help("Define custom workspace location"),
                    )
                    .next_help_heading("Output")
                    .arg(Arg::new("output").short('o').help(
                        "Customize output file name. Otherwise, we use the same station name.",
                    ))
                    .arg(
                        Arg::new("gzip")
                            .short('g')
                            .help("Activate Gzip compression. Output is gzip compressed.")
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("gzip-lvl").short('c').help(
                            "Set custom gzip compression level, 1 <= lvl <= 9, default is 6.",
                        ),
                    )
                    .get_matches()
            },
        }
    }
    pub fn input_path(&self) -> PathBuf {
        Path::new(self.matches.get_one::<String>("input").unwrap()).to_path_buf()
    }
    pub fn output_name(&self) -> Option<&String> {
        self.matches.get_one::<String>("output")
    }
    pub fn workspace(&self) -> Option<&String> {
        self.matches.get_one::<String>("workspace")
    }
    pub fn gzip(&self) -> bool {
        self.matches.get_flag("gzip")
    }
    pub fn gzip_compression_lvl(&self) -> u32 {
        if let Some(lvl) = self.matches.get_one::<String>("gzip-lvl") {
            lvl.parse::<u32>().unwrap_or(6)
        } else {
            6
        }
    }
}
