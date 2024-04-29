/* Features that only get activated on "qc" option */
use crate::prelude::Rinex;
use hifitime::TimeSeries;
use rinex_qc_traits::{MaskFilter, Masking};

use crate::{
    epoch::epoch_decompose,
    prelude::{Constellation, Duration, Epoch, RinexType},
};

// RINEX production infrastructure // physical observations
pub(crate) mod production;
pub use production::{DataSource, DetailedProductionAttributes, ProductionAttributes, FFU, PPU};

#[cfg_attr(docrs, doc(cfg(feature = "qc")))]
impl Rinex {
    /// Iterate over unexpected data gaps,
    /// in the form ([`Epoch`], [`Duration`]), where
    /// epoch is the datetime where a gap started, and its duration.
    /// ```
    /// use std::str::FromStr;
    /// use rinex::prelude::{Rinex, Epoch, Duration};
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    ///
    /// // when tolerance is set to None,
    /// // the reference sample rate is [Self::dominant_sample_rate].
    /// let mut tolerance : Option<Duration> = None;
    /// let gaps : Vec<_> = rinex.data_gaps(tolerance).collect();
    /// assert!(
    ///     rinex.data_gaps(None).eq(
    ///         vec![
    ///             (Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(), Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(), Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(), Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T23:02:00 UTC").unwrap(), Duration::from_seconds(7.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T23:21:00 UTC").unwrap(), Duration::from_seconds(31.0 * 60.0)),
    ///         ]),
    ///     "data_gaps(tol=None) failed"
    /// );
    ///
    /// // with a tolerance, we tolerate the given gap duration
    /// tolerance = Some(Duration::from_seconds(3600.0));
    /// let gaps : Vec<_> = rinex.data_gaps(tolerance).collect();
    /// assert!(
    ///     rinex.data_gaps(Some(Duration::from_seconds(3.0 * 3600.0))).eq(
    ///         vec![
    ///             (Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(), Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(), Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(), Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)),
    ///         ]),
    ///     "data_gaps(tol=3h) failed"
    /// );
    /// ```
    pub fn data_gaps(
        &self,
        tolerance: Option<Duration>,
    ) -> Box<dyn Iterator<Item = (Epoch, Duration)> + '_> {
        let sample_rate: Duration = match tolerance {
            Some(dt) => dt, // user defined
            None => {
                match self.dominant_sample_rate() {
                    Some(dt) => dt,
                    None => {
                        match self.sample_rate() {
                            Some(dt) => dt,
                            None => {
                                // not enough information
                                // this is probably not an Epoch iterated RINEX
                                return Box::new(Vec::<(Epoch, Duration)>::new().into_iter());
                            },
                        }
                    },
                }
            },
        };
        Box::new(
            self.epoch()
                .zip(self.epoch().skip(1))
                .filter_map(move |(ek, ekp1)| {
                    let dt = ekp1 - ek; // gap
                    if dt > sample_rate {
                        // too large
                        Some((ek, dt)) // retain starting datetime and gap duration
                    } else {
                        None
                    }
                }),
        )
    }
    /// Returns dominant sample rate, ie., most common [Duration] between successive
    /// [Epoch]s, discarding minor data gaps that may have occurred.
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// assert_eq!(
    ///     rnx.dominant_sample_rate(),
    ///     Some(Duration::from_seconds(60.0)));
    /// ```
    pub fn dominant_sample_rate(&self) -> Option<Duration> {
        self.sampling_histogram()
            .max_by(|(_, pop_i), (_, pop_j)| pop_i.cmp(pop_j))
            .map(|dominant| dominant.0)
    }
    /// Guesses File [ProductionAttributes] from the actual Record content.
    /// This is particularly useful when working with datasets we are confident about,
    /// yet that do not follow standard naming conventions.
    /// Here is an example of such use case:
    /// ```
    /// use rinex::prelude::*;
    ///
    /// // Parse one file that does not follow naming conventions
    /// let rinex = Rinex::from_file("../test_resources/MET/V4/example1.txt");
    /// assert!(rinex.is_ok()); // As previously stated, we totally accept that
    /// let rinex = rinex.unwrap();
    ///
    /// // The standard file name generator has no means to generate something correct.
    /// let standard_name = rinex.standard_filename(true, None, None);
    /// assert_eq!(standard_name, "XXXX0070.21M");
    ///
    /// // We use the smart attributes detector as custom attributes
    /// let guessed = rinex.guess_production_attributes();
    /// let standard_name = rinex.standard_filename(true, None, Some(guessed.clone()));
    ///
    /// // we get a perfect shortened name
    /// assert_eq!(standard_name, "bako0070.21M");
    ///
    /// // If we ask for a (modern) long standard filename, we mostly get it right,
    /// // but some fields like the Country code can only be determined from the original filename,
    /// // so we have no means to receover them.
    /// let standard_name = rinex.standard_filename(false, None, Some(guessed.clone()));
    /// assert_eq!(standard_name, "bako00XXX_U_20210070000_00U_MM.rnx");
    /// ```
    pub fn guess_production_attributes(&self) -> ProductionAttributes {
        // start from content identified from the filename
        let mut attributes = self.prod_attr.clone().unwrap_or_default();

        let first_epoch = self.first_epoch();
        let last_epoch = self.last_epoch();
        let first_epoch_gregorian = first_epoch.map(|t0| t0.to_gregorian_utc());

        match first_epoch_gregorian {
            Some((y, _, _, _, _, _, _)) => attributes.year = y as u32,
            _ => {},
        }
        match first_epoch {
            Some(t0) => attributes.doy = t0.day_of_year().round() as u32,
            _ => {},
        }
        // notes on attribute."name"
        // - Non detailed OBS RINEX: this is usually the station name
        //   which can be named after a geodetic marker
        // - Non detailed NAV RINEX: station name
        // - CLK RINEX: name of the local clock
        // - IONEX: agency
        match self.header.rinex_type {
            RinexType::ClockData => match &self.header.clock {
                Some(clk) => match &clk.ref_clock {
                    Some(refclock) => attributes.name = refclock.to_string(),
                    _ => {
                        if let Some(site) = &clk.site {
                            attributes.name = site.to_string();
                        } else {
                            attributes.name = self.header.agency.to_string();
                        }
                    },
                },
                _ => attributes.name = self.header.agency.to_string(),
            },
            RinexType::IonosphereMaps => {
                attributes.name = self.header.agency.to_string();
            },
            _ => match &self.header.geodetic_marker {
                Some(marker) => attributes.name = marker.name.to_string(),
                _ => attributes.name = self.header.agency.to_string(),
            },
        }
        if let Some(ref mut details) = attributes.details {
            if let Some((_, _, _, hh, mm, _, _)) = first_epoch_gregorian {
                details.hh = hh;
                details.mm = mm;
            }
            if let Some(first_epoch) = first_epoch {
                if let Some(last_epoch) = last_epoch {
                    let total_dt = last_epoch - first_epoch;
                    details.ppu = PPU::from(total_dt);
                }
            }
        } else {
            attributes.details = Some(DetailedProductionAttributes {
                batch: 0,                      // see notes down below
                country: "XXX".to_string(),    // see notes down below
                data_src: DataSource::Unknown, // see notes down below
                ppu: match (first_epoch, last_epoch) {
                    (Some(first), Some(last)) => {
                        let total_dt = last - first;
                        PPU::from(total_dt)
                    },
                    _ => PPU::Unspecified,
                },
                ffu: self.dominant_sample_rate().map(FFU::from),
                hh: match first_epoch_gregorian {
                    Some((_, _, _, hh, _, _, _)) => hh,
                    _ => 0,
                },
                mm: match first_epoch_gregorian {
                    Some((_, _, _, _, mm, _, _)) => mm,
                    _ => 0,
                },
            });
        }
        /*
         * Several fields cannot be deduced from the actual
         * Record content. If provided filename did not describe them,
         * we have no means to recover them.
         * Example of such fields would be:
         *    + Country Code: would require a worldwide country database
         *    + Data source: is only defined in the filename
         */
        attributes
    }
    /// Returns a filename that would describe Self according to standard naming conventions.
    /// For this information to be 100% complete, Self must come from a file
    /// that follows these conventions itself.
    /// Otherwise you must provide [ProductionAttributes] yourself with "custom".
    /// In any case, this method is infaillible. You will just lack more or
    /// less information, depending on current context.
    /// If you're working with Observation, Navigation or Meteo data,
    /// and prefered shorter filenames (V2 like format): force short to "true".
    /// Otherwse, we will prefer modern V3 like formats.
    /// Use "suffix" to append a custom suffix like ".gz" for example.
    /// NB this will only output uppercase filenames (as per standard specs).
    /// ```
    /// use rinex::prelude::*;
    /// // Parse a File that follows standard naming conventions
    /// // and verify we generate something correct
    /// ```
    pub fn standard_filename(
        &self,
        short: bool,
        suffix: Option<&str>,
        custom: Option<ProductionAttributes>,
    ) -> String {
        let header = &self.header;
        let rinextype = header.rinex_type;
        let is_crinex = header.is_crinex();
        let constellation = header.constellation;

        let mut filename = match rinextype {
            RinexType::IonosphereMaps => {
                let name = match custom {
                    Some(ref custom) => {
                        custom.name[..std::cmp::min(3, custom.name.len())].to_string()
                    },
                    None => {
                        if let Some(attr) = &self.prod_attr {
                            attr.name.clone()
                        } else {
                            "XXX".to_string()
                        }
                    },
                };
                let region = match &custom {
                    Some(ref custom) => custom.region.unwrap_or('G'),
                    None => {
                        if let Some(attr) = &self.prod_attr {
                            attr.region.unwrap_or('G')
                        } else {
                            'G'
                        }
                    },
                };
                let ddd = match &custom {
                    Some(ref custom) => format!("{:03}", custom.doy),
                    None => {
                        if let Some(epoch) = self.first_epoch() {
                            let ddd = epoch.day_of_year().round() as u32;
                            format!("{:03}", ddd)
                        } else {
                            "DDD".to_string()
                        }
                    },
                };
                let yy = match &custom {
                    Some(ref custom) => format!("{:02}", custom.year - 2_000),
                    None => {
                        if let Some(epoch) = self.first_epoch() {
                            let yy = epoch_decompose(epoch).0;
                            format!("{:02}", yy - 2_000)
                        } else {
                            "YY".to_string()
                        }
                    },
                };
                ProductionAttributes::ionex_format(&name, region, &ddd, &yy)
            },
            RinexType::ObservationData | RinexType::MeteoData | RinexType::NavigationData => {
                let name = match custom {
                    Some(ref custom) => custom.name.clone(),
                    None => {
                        if let Some(attr) = &self.prod_attr {
                            attr.name.clone()
                        } else {
                            "XXXX".to_string()
                        }
                    },
                };
                let ddd = match &custom {
                    Some(ref custom) => format!("{:03}", custom.doy),
                    None => {
                        if let Some(epoch) = self.first_epoch() {
                            let ddd = epoch.day_of_year().round() as u32;
                            format!("{:03}", ddd)
                        } else {
                            "DDD".to_string()
                        }
                    },
                };
                if short {
                    let yy = match &custom {
                        Some(ref custom) => format!("{:02}", custom.year - 2_000),
                        None => {
                            if let Some(epoch) = self.first_epoch() {
                                let yy = epoch_decompose(epoch).0;
                                format!("{:02}", yy - 2_000)
                            } else {
                                "YY".to_string()
                            }
                        },
                    };
                    let ext = match rinextype {
                        RinexType::ObservationData => {
                            if is_crinex {
                                'D'
                            } else {
                                'O'
                            }
                        },
                        RinexType::MeteoData => 'M',
                        RinexType::NavigationData => match constellation {
                            Some(Constellation::Glonass) => 'G',
                            _ => 'N',
                        },
                        _ => unreachable!("unreachable"),
                    };
                    ProductionAttributes::rinex_short_format(&name, &ddd, &yy, ext)
                } else {
                    /* long /V3 like format */
                    let batch = match &custom {
                        Some(ref custom) => {
                            if let Some(details) = &custom.details {
                                details.batch
                            } else {
                                0
                            }
                        },
                        None => {
                            if let Some(attr) = &self.prod_attr {
                                if let Some(details) = &attr.details {
                                    details.batch
                                } else {
                                    0
                                }
                            } else {
                                0
                            }
                        },
                    };
                    let country = match &custom {
                        Some(ref custom) => {
                            if let Some(details) = &custom.details {
                                details.country.to_string()
                            } else {
                                "CCC".to_string()
                            }
                        },
                        None => {
                            if let Some(attr) = &self.prod_attr {
                                if let Some(details) = &attr.details {
                                    details.country.to_string()
                                } else {
                                    "CCC".to_string()
                                }
                            } else {
                                "CCC".to_string()
                            }
                        },
                    };
                    let src = match &header.rcvr {
                        Some(_) => 'R', // means GNSS rcvr
                        None => {
                            if let Some(attr) = &self.prod_attr {
                                if let Some(details) = &attr.details {
                                    details.data_src.to_char()
                                } else {
                                    'U' // means unspecified
                                }
                            } else {
                                'U' // means unspecified
                            }
                        },
                    };
                    let yyyy = match &custom {
                        Some(ref custom) => format!("{:04}", custom.year),
                        None => {
                            if let Some(epoch) = self.first_epoch() {
                                let yy = epoch_decompose(epoch).0;
                                format!("{:04}", yy)
                            } else {
                                "YYYY".to_string()
                            }
                        },
                    };
                    let (hh, mm) = match &custom {
                        Some(ref custom) => {
                            if let Some(details) = &custom.details {
                                (format!("{:02}", details.hh), format!("{:02}", details.mm))
                            } else {
                                ("HH".to_string(), "MM".to_string())
                            }
                        },
                        None => {
                            if let Some(epoch) = self.first_epoch() {
                                let (_, _, _, hh, mm, _, _) = epoch_decompose(epoch);
                                (format!("{:02}", hh), format!("{:02}", mm))
                            } else {
                                ("HH".to_string(), "MM".to_string())
                            }
                        },
                    };
                    // FFU sampling rate
                    let ffu = match self.dominant_sample_rate() {
                        Some(duration) => FFU::from(duration).to_string(),
                        None => {
                            if let Some(ref custom) = custom {
                                if let Some(details) = &custom.details {
                                    if let Some(ffu) = details.ffu {
                                        ffu.to_string()
                                    } else {
                                        "XXX".to_string()
                                    }
                                } else {
                                    "XXX".to_string()
                                }
                            } else {
                                "XXX".to_string()
                            }
                        },
                    };
                    // ffu only in OBS file names
                    let ffu = match rinextype {
                        RinexType::ObservationData => Some(ffu),
                        _ => None,
                    };
                    // PPU periodicity
                    let ppu = if let Some(ref custom) = custom {
                        if let Some(details) = &custom.details {
                            details.ppu
                        } else {
                            PPU::Unspecified
                        }
                    } else if let Some(ref attr) = self.prod_attr {
                        if let Some(details) = &attr.details {
                            details.ppu
                        } else {
                            PPU::Unspecified
                        }
                    } else {
                        PPU::Unspecified
                    };
                    let fmt = match rinextype {
                        RinexType::ObservationData => "MO".to_string(),
                        RinexType::MeteoData => "MM".to_string(),
                        RinexType::NavigationData => match constellation {
                            Some(Constellation::Mixed) | None => "MN".to_string(),
                            Some(constell) => format!("M{:x}", constell),
                        },
                        _ => unreachable!("unreachable fmt"),
                    };
                    let ext = if is_crinex { "crx" } else { "rnx" };
                    ProductionAttributes::rinex_long_format(
                        &name,
                        batch,
                        &country,
                        src,
                        &yyyy,
                        &ddd,
                        &hh,
                        &mm,
                        &ppu.to_string(),
                        ffu.as_deref(),
                        &fmt,
                        ext,
                    )
                }
            },
            rinex => unimplemented!("{} format", rinex),
        };
        if let Some(suffix) = suffix {
            filename.push_str(suffix);
        }
        filename
    }
    /// Returns True if Self has a steady sampling, ie., made of evenly spaced [Epoch].
    pub fn steady_sampling(&self) -> bool {
        self.sampling_histogram().count() == 1
    }
    /// Forms a [`TimeSeries`] iterator spanning [Self::duration]
    /// and dt=[Self::dominant_sample_rate].
    pub fn timeseries(&self) -> Option<TimeSeries> {
        let start = self.first_epoch()?;
        let end = self.last_epoch()?;
        let dt = self.dominant_sample_rate()?;
        Some(TimeSeries::inclusive(start, end, dt))
    }
}

#[cfg(feature = "qc")]
#[cfg_attr(docrs, doc(cfg(feature = "qc")))]
impl Masking for Rinex {
    fn mask(&self, f: &MaskFilter) -> Self {
        let mut s = self.clone();
        s.mask_mut(f);
        s
    }
    fn mask_mut(&mut self, f: &MaskFilter) {
        self.record.mask_mut(f);
        self.header.mask_mut(f);
    }
}
