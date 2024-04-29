#[cfg(test)]
mod test {
    use crate::{
        erratic_time_frame, evenly_spaced_time_frame,
        marker::MarkerType,
        observable,
        observation::SNR,
        prelude::{
            Constellation, Duration, Epoch, EpochFlag, GroundPosition, Header, LliFlags,
            Observable, Rinex, SV,
        },
        tests::toolkit::{obsrinex_check_observables, test_observation_rinex, TestTimeFrame},
        version::Version,
        Carrier,
    };
    use gnss_rs::sv;
    use hifitime::TimeSeries;
    use itertools::Itertools;
    use std::path::Path;
    use std::str::FromStr;
    #[test]
    fn v2_aopr0010_17o() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V2")
            .join("aopr0010.17o");
        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_file(fullpath.as_ref());
        assert!(rinex.is_ok());
        let rinex = rinex.unwrap();

        test_observation_rinex(
            &rinex,
            "2.10",
            Some("GPS"),
            "GPS",
            "G01,G06,G09,G31,G17,G26,G28,G27,G03,G32,G16,G14,G08,G23,G22,G07, G30, G11, G19, G07",
            "C1, L1, L2, P2, P1",
            Some("2017-01-01T00:00:00 GPST"),
            None,
            erratic_time_frame!(
                "2017-01-01T00:00:00 GPST,
                2017-01-01T03:33:40 GPST,
                2017-01-01T06:09:10 GPST"
            ),
        );

        /* This file is GPS */
        obsrinex_check_observables(&rinex, Constellation::GPS, &["L1", "L2", "C1", "P1", "P2"]);

        let record = rinex.record.as_obs().unwrap();
        for (k, v) in record.iter() {
            assert!(k.flag.is_ok(), "bad epoch flag @{:?}", k.epoch);
            assert!(v.clock_offset.is_none(), "bad clock offset @{:?}", k.epoch);
            let formatted_e = format!("{:?}", k.epoch);
            for (k, obs_data) in v.observations.iter() {
                let formatted_obs = k.observable.to_string();
                match formatted_e.as_str() {
                    "2017-01-01T00:00:00 GPST" => match formatted_obs.as_str() {
                        "P1" => match k.sv.constellation {
                            Constellation::GPS => match k.sv.prn {
                                31 => assert_eq!(obs_data.value, 22513484.772),
                                26 => assert_eq!(obs_data.value, 21540206.156),
                                _ => {},
                            },
                            _ => panic!("should not exist"),
                        },
                        "P2" => match k.sv.constellation {
                            Constellation::GPS => match k.sv.prn {
                                31 => assert_eq!(obs_data.value, 22513487.370),
                                26 => assert_eq!(obs_data.value, 21540211.941),
                                _ => {},
                            },
                            _ => panic!("should not exist"),
                        },
                        _ => {},
                    },
                    "2017-01-01T03:33:40 GPST" => match formatted_obs.as_str() {
                        "P1" => match k.sv.constellation {
                            Constellation::GPS => match k.sv.prn {
                                31 => assert_eq!(obs_data.value, 24352347.924),
                                26 => assert_eq!(obs_data.value, 21708306.570),
                                _ => {},
                            },
                            _ => panic!("should not exist"),
                        },
                        "P2" => match k.sv.constellation {
                            Constellation::GPS => match k.sv.prn {
                                31 => assert_eq!(obs_data.value, 24352356.156),
                                26 => assert_eq!(obs_data.value, 21708312.941),
                                _ => {},
                            },
                            _ => panic!("should not exist"),
                        },
                        _ => {},
                    },
                    "2017-01-01T06:09:10 GPST" => match formatted_obs.as_str() {
                        "P1" => match k.sv.constellation {
                            Constellation::GPS => match k.sv.prn {
                                31 => assert_eq!(obs_data.value, 20796244.823),
                                26 => assert_eq!(obs_data.value, 21184456.914),
                                _ => {},
                            },
                            _ => panic!("should not exist"),
                        },
                        "P2" => match k.sv.constellation {
                            Constellation::GPS => match k.sv.prn {
                                31 => assert_eq!(obs_data.value, 20796250.633),
                                26 => assert_eq!(obs_data.value, 21184462.122),
                                _ => {},
                            },
                            _ => panic!("should not exist"),
                        },
                        _ => {},
                    },
                    _ => panic!("should not exist"),
                }
            }
        }
    }
    #[test]
    fn v2_npaz3550_21o() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V2")
            .join("npaz3550.21o");
        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_file(fullpath.as_ref());
        assert!(rinex.is_ok());
        let rinex = rinex.unwrap();

        test_observation_rinex(
            &rinex,
            "2.11",
            Some("MIXED"),
            "GPS, GLO",
            "G01,G08,G10,G15,G16,G18,G21,G23,G26,G32,R04,R05,R06,R07,R10,R12,R19,R20,R21,R22",
            "C1, L1, L2, P2, S1, S2",
            Some("2021-12-21T00:00:00 GPST"),
            Some("2021-12-21T23:59:30 GPST"),
            evenly_spaced_time_frame!(
                "2021-12-21T00:00:00 GPST",
                "2021-12-21T01:04:00 GPST",
                "30 s"
            ),
        );

        /* This file is GPS + GLO */
        obsrinex_check_observables(
            &rinex,
            Constellation::GPS,
            &["C1", "L1", "L2", "P2", "S1", "S2"],
        );
        obsrinex_check_observables(
            &rinex,
            Constellation::Glonass,
            &["C1", "L1", "L2", "P2", "S1", "S2"],
        );

        let record = rinex.record.as_obs().unwrap();
        for (k, v) in record.iter() {
            assert!(k.flag.is_ok(), "bad epoch flag @{:?}", k.epoch);
            assert!(v.clock_offset.is_none(), "bad clock offset @{:?}", k.epoch);
            for (k, obs_data) in v.observations.iter() {
                //TODO add more tests
            }
        }
    }
    #[test]
    fn v2_rovn0010_21o() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V2")
            .join("rovn0010.21o");
        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_file(fullpath.as_ref());
        assert!(rinex.is_ok());
        let rinex = rinex.unwrap();

        test_observation_rinex(
            &rinex,
            "2.11",
            Some("MIXED"),
            "GPS, GLO",
            "G07,G08,G10,G13,G15,G16,G18,G20,G21,G23,G26,G27,
G30,R01,R02,R03,R08,R09,R15,R16,R17,R18,R19,R24,
G07,G08,G10,G13,G15,G16,G18,G20,G21,G23,G26,G27,G30,R01,R02,
R03,R08,R09,R15,R16,R17,R18,R19,R24,G01,G07,G08,G10,G14,G15,
G16,G20,G21,G22,G23,G27,G28,G30,G32,R01,R02,R03,R04,R09,R10,R16,R17,R18,R19,
G01,G03,G08,G10,G14,G21,G22,G24,G27,G28,G32,R02,
R03,R04,R09,R10,R17,R18,R19,R20,
G01,G03,G08,G10,G14,G21,G22,G24,G27,G28,G32,R02,
R03,R04,R09,R10,R17,R18,R19,R20,
G01,G03,G08,G10,G14,G21,G22,G24,G27,G28,G32,R02,
R03,R04,R09,R10,R17,R18,R19,R20",
            "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5",
            Some("2021-01-01T00:00:00 GPST"),
            Some("2021-01-01T23:59:30 GPST"),
            erratic_time_frame!(
                "
                2021-01-01T00:00:00 GPST,
                2021-01-01T00:00:30 GPST,
                2021-01-01T01:10:00 GPST,
                2021-01-01T02:25:00 GPST,
                2021-01-01T02:25:30 GPST,
                2021-01-01T02:26:00 GPST
            "
            ),
        );

        /* This file is GPS + GLO */
        obsrinex_check_observables(
            &rinex,
            Constellation::GPS,
            &[
                "C1", "C2", "C5", "L1", "L2", "L5", "P1", "P2", "S1", "S2", "S5",
            ],
        );

        obsrinex_check_observables(
            &rinex,
            Constellation::Glonass,
            &[
                "C1", "C2", "C5", "L1", "L2", "L5", "P1", "P2", "S1", "S2", "S5",
            ],
        );

        /*
         * Header tb
         */
        let header = &rinex.header;
        assert_eq!(
            header.ground_position,
            Some(GroundPosition::from_ecef_wgs84((
                3859571.8076,
                413007.6749,
                5044091.5729
            )))
        );

        let marker = &header.geodetic_marker;
        assert!(marker.is_some(), "failed to parse geodetic marker");
        let marker = marker.as_ref().unwrap();
        assert_eq!(marker.number(), Some("13544M001".to_string()));
        assert_eq!(header.observer, "Hans van der Marel");
        assert_eq!(header.agency, "TU Delft for Deltares");

        let record = rinex.record.as_obs().unwrap();
        for (k, v) in record.iter() {
            assert!(k.flag.is_ok(), "bad epoch flag @{:?}", k.epoch);
            assert!(v.clock_offset.is_none(), "bad clock offset @{:?}", k.epoch);
            for (k, obs_data) in v.observations.iter() {
                //TODO add more tests
            }
        }
    }
    #[test]
    fn v3_duth0630() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V3")
            .join("DUTH0630.22O");
        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_file(fullpath.as_ref());
        assert!(rinex.is_ok());
        let rinex = rinex.unwrap();

        test_observation_rinex(
            &rinex,
            "3.02",
            Some("MIXED"),
            "GPS, GLO",
            "G03, G01, G04, G09, G06, G17, G26, G19, G21, G22, G31, G32, R01, R02, R08, R09, R10, R17, R23, R24",
            "C1C, L1C, D1C, S1C, C2P, L2P, D2P, S2P, C2W, L2W, D2W, S2W",
            Some("2022-03-04T00:00:00 GPST"),
            Some("2022-03-04T23:59:30 GPST"),
            erratic_time_frame!(
                "2022-03-04T00:00:00 GPST, 2022-03-04T00:28:30 GPST, 2022-03-04T00:57:00 GPST"
            ),
        );

        /* This file is G + R */
        obsrinex_check_observables(
            &rinex,
            Constellation::GPS,
            &["C1C", "L1C", "D1C", "S1C", "C2W", "L2W", "D2W", "S2W"],
        );
        obsrinex_check_observables(
            &rinex,
            Constellation::Glonass,
            &["C1C", "L1C", "D1C", "S1C", "C2P", "L2P", "D2P", "S2P"],
        );

        let record = rinex.record.as_obs().unwrap();
        for (k, v) in record.iter() {
            assert!(k.flag.is_ok(), "bad epoch flag @{:?}", k.epoch);
            assert!(v.clock_offset.is_none(), "bad clock offset @{:?}", k.epoch);
            for (k, obs_data) in v.observations.iter() {
                //TODO add more tests
            }
        }
    }
    #[test]
    fn v4_kms300dnk_r_2022_v3crx() {
        let test_resource = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/CRNX/V3/KMS300DNK_R_20221591000_01H_30S_MO.crx";
        let rinex = Rinex::from_file(&test_resource);
        assert!(rinex.is_ok());
        let rinex = rinex.unwrap();
        //////////////////////////
        // Header testbench
        //////////////////////////
        assert!(rinex.is_observation_rinex());
        assert!(rinex.header.obs.is_some());

        /* this file is G +E +R +J +S +C */
        obsrinex_check_observables(
            &rinex,
            Constellation::BeiDou,
            &[
                "C1P", "C2I", "C5P", "C6I", "C7D", "C7I", "L1P", "L2I", "L5P", "L6I", "L7D", "L7I",
            ],
        );

        obsrinex_check_observables(
            &rinex,
            Constellation::Galileo,
            &[
                "C1C", "C5Q", "C6C", "C7Q", "C8Q", "L1C", "L5Q", "L6C", "L7Q", "L8Q",
            ],
        );

        obsrinex_check_observables(
            &rinex,
            Constellation::GPS,
            &[
                "C1C", "C1L", "C1W", "C2L", "C2W", "C5Q", "L1C", "L1L", "L2L", "L2W", "L5Q",
            ],
        );

        obsrinex_check_observables(
            &rinex,
            Constellation::QZSS,
            &["C1C", "C1L", "C2L", "C5Q", "L1C", "L1L", "L2L", "L5Q"],
        );

        obsrinex_check_observables(
            &rinex,
            Constellation::Glonass,
            &[
                "C1C", "C1P", "C2C", "C2P", "C3Q", "L1C", "L1P", "L2C", "L2P", "L3Q",
            ],
        );

        obsrinex_check_observables(&rinex, Constellation::SBAS, &["C1C", "C5I", "L1C", "L5I"]);

        let record = rinex.record.as_obs().unwrap();

        for (k, v) in record.iter() {
            assert!(k.flag.is_ok(), "bad epoch flag @{:?}", k.epoch);
            assert!(v.clock_offset.is_none(), "bad clock offset @{:?}", k.epoch);
            for (k, obs_data) in v.observations.iter() {
                // TODO add more tests
            }
        }
    }
    #[test]
    fn v2_kosg0010_95o() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V2")
            .join("KOSG0010.95O");
        let fullpath = path.to_string_lossy();
        let rnx = Rinex::from_file(fullpath.as_ref()).unwrap();
        test_observation_rinex(
            &rnx,
            "2.0",
            Some("GPS"),
            "GPS",
            "G01, G04, G05, G06, G16, G17, G18, G19, G20, G21, G22, G23, G24, G25, G27, G29, G28, G31",
            "L1, L2, P1, P2, C1",
            Some("1995-01-01T00:00:00 GPST"),
            Some("1995-01-01T23:59:30 GPST"),
            erratic_time_frame!(
                "
                1995-01-01T00:00:00 GPST,
                1995-01-01T11:00:00 GPST,
                1995-01-01T20:44:30 GPST
            "
            ),
        );
    }
    #[test]
    fn v2_ajac3550() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V2")
            .join("AJAC3550.21O");
        let fullpath = path.to_string_lossy();
        let rnx = Rinex::from_file(fullpath.as_ref()).unwrap();
        let epochs: Vec<Epoch> = vec![
            Epoch::from_str("2021-12-21T00:00:00 GPST").unwrap(),
            Epoch::from_str("2021-12-21T00:00:30 GPST").unwrap(),
        ];

        // Check parsed observables
        for constellation in [
            Constellation::GPS,
            Constellation::SBAS,
            Constellation::Glonass,
            Constellation::Galileo,
        ] {
            obsrinex_check_observables(
                &rnx,
                constellation,
                &[
                    "L1", "L2", "C1", "C2", "P1", "P2", "D1", "D2", "S1", "S2", "L5", "C5", "D5",
                    "S5", "L7", "C7", "D7", "S7", "L8", "C8", "D8", "S8",
                ],
            );
        }

        assert_eq!(
            rnx.epoch().collect::<Vec<Epoch>>(),
            epochs,
            "parsed wrong epoch content"
        );

        let freqz = rnx.carrier().sorted().collect::<Vec<_>>();
        assert_eq!(
            freqz,
            ["L1", "L2", "L5", "E1", "E5", "G1", "G2"]
                .iter()
                .map(|c| Carrier::from_str(c).unwrap())
                .sorted()
                .collect::<Vec<_>>(),
        );

        let phase_l1c: Vec<_> = rnx
            .carrier_phase()
            .filter_map(|(e, f, sv, obs, value)| {
                if *obs == observable!("L1") {
                    Some((e, f, sv, value))
                } else {
                    None
                }
            })
            .collect();

        assert!(phase_l1c.len() > 0, "failed to form l1 phase iterator");

        for (t, flag, sv, value) in phase_l1c {
            assert!(flag.is_ok(), "bad L1 flag");
            let formatted = format!("{:?}", t);
            match formatted.as_str() {
                "2021-12-21T00:00:00 GPST" => match sv.constellation {
                    Constellation::GPS => match sv.prn {
                        7 => assert_eq!(value, 131857102.133),
                        8 => assert_eq!(value, 114374313.914),
                        10 => assert_eq!(value, 110158976.908),
                        16 => assert_eq!(value, 112191307.034),
                        _ => {},
                    },
                    _ => {},
                },
                "2021-12-21T00:00:30 GPST" => match sv.constellation {
                    Constellation::GPS => match sv.prn {
                        7 => assert_eq!(value, 131869667.223),
                        _ => {},
                    },
                    Constellation::Galileo => match sv.prn {
                        31 => assert_eq!(value, 145142607.255),
                        33 => assert_eq!(value, 142356780.263),
                        _ => {},
                    },
                    Constellation::SBAS => match sv.prn {
                        23 => assert_eq!(value, 200051837.090),
                        36 => assert_eq!(value, 197948874.430),
                        _ => {},
                    },
                    _ => {},
                },
                _ => {},
            }
        }

        let pr_c1: Vec<_> = rnx
            .pseudo_range()
            .filter_map(|(e, f, sv, obs, value)| {
                if *obs == observable!("C1") {
                    Some((e, f, sv, value))
                } else {
                    None
                }
            })
            .collect();

        assert!(pr_c1.len() > 0, "failed to form c1 pr iterator");
        for (t, flag, sv, value) in pr_c1 {
            assert!(flag.is_ok(), "bad L1 flag");
            let formatted = format!("{:?}", t);
            match formatted.as_str() {
                "2021-12-21T00:00:00 GPST" => match sv.constellation {
                    Constellation::GPS => match sv.prn {
                        7 => assert_eq!(value, 25091572.300),
                        8 => assert_eq!(value, 21764705.880),
                        10 => assert_eq!(value, 20962551.380),
                        16 => assert_eq!(value, 21349295.460),
                        _ => {},
                    },
                    _ => {},
                },
                "2021-12-21T00:00:30 GPST" => match sv.constellation {
                    Constellation::GPS => match sv.prn {
                        7 => assert_eq!(value, 25093963.200),
                        8 => assert_eq!(value, 21751524.200),
                        10 => assert_eq!(value, 20960258.720),
                        16 => assert_eq!(value, 21362073.640),
                        _ => {},
                    },
                    _ => {},
                },
                _ => {},
            }
        }

        let record = rnx.record.as_obs().unwrap();

        for (k, v) in record.iter() {
            assert!(k.flag.is_ok(), "bad epoch flag @{:?}", k.epoch);
            assert!(v.clock_offset.is_none(), "bad clock offset @{:?}", k.epoch);
            let formatted_e = format!("{:?}", k.epoch);
            for (k, obs_data) in v.observations.iter() {
                let formatted_obs = k.observable.to_string();
                match formatted_e.as_str() {
                    "2021-12-21T00:00:00 GPST" => match formatted_obs.as_str() {
                        "D1" => match k.sv.constellation {
                            Constellation::GPS => match k.sv.prn {
                                7 => assert_eq!(obs_data.value, -411.138),
                                8 => assert_eq!(obs_data.value, 2312.498),
                                10 => assert_eq!(obs_data.value, 410.954),
                                _ => {},
                            },
                            Constellation::Galileo => match k.sv.prn {
                                31 => assert_eq!(obs_data.value, -3209.037),
                                33 => assert_eq!(obs_data.value, -2190.415),
                                _ => {},
                            },
                            Constellation::SBAS => match k.sv.prn {
                                23 => assert_eq!(obs_data.value, 2.966),
                                36 => assert_eq!(obs_data.value, -1.295),
                                _ => {},
                            },
                            _ => {},
                        },
                        "D2" => match k.sv.constellation {
                            Constellation::GPS => match k.sv.prn {
                                7 => assert_eq!(obs_data.value, -320.373),
                                8 => assert_eq!(obs_data.value, 1801.947),
                                10 => assert_eq!(obs_data.value, 320.225),
                                _ => {},
                            },
                            Constellation::SBAS => match k.sv.prn {
                                36 => panic!("should not exist!"),
                                23 => panic!("should not exist!"),
                                _ => {},
                            },
                            Constellation::Galileo => match k.sv.prn {
                                33 => panic!("should not exist!"),
                                31 => panic!("should not exist!"),
                                _ => {},
                            },
                            _ => {},
                        },
                        _ => {},
                    },
                    "2021-12-21T00:00:30 GPST" => match formatted_e.as_str() {
                        "D2" => match k.sv.constellation {
                            Constellation::GPS => match k.sv.prn {
                                7 => assert_eq!(obs_data.value, -426.868),
                                8 => assert_eq!(obs_data.value, 2305.387),
                                10 => assert_eq!(obs_data.value, 391.975),
                                _ => {},
                            },
                            Constellation::SBAS => match k.sv.prn {
                                31 => panic!("should not exist"),
                                33 => panic!("should not exist"),
                                _ => {},
                            },
                            _ => {},
                        },
                        _ => {},
                    },
                    _ => {},
                }
            }
        }
    }
    #[test]
    fn v3_noa10630() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V3")
            .join("NOA10630.22O");
        let fullpath = path.to_string_lossy();
        let rnx = Rinex::from_file(fullpath.as_ref()).unwrap();

        test_observation_rinex(
            &rnx,
            "3.02",
            Some("GPS"),
            "GPS",
            "G01, G03, G04, G06, G09, G17, G19, G21, G22, G31",
            "C1C, L1C, D1C, S1C, S2W, L2W, D2W, S2W",
            Some("2022-03-04T00:00:00 GPST"),
            Some("2022-03-04T23:59:30 GPST"),
            erratic_time_frame!(
                "
                2022-03-04T00:00:00 GPST,
                2022-03-04T00:00:30 GPST,
                2022-03-04T00:01:00 GPST,
                2022-03-04T00:52:30 GPST"
            ),
        );

        let expected: Vec<Epoch> = vec![
            Epoch::from_str("2022-03-04T00:00:00 GPST").unwrap(),
            Epoch::from_str("2022-03-04T00:00:30 GPST").unwrap(),
            Epoch::from_str("2022-03-04T00:01:00 GPST").unwrap(),
            Epoch::from_str("2022-03-04T00:52:30 GPST").unwrap(),
        ];
        assert_eq!(
            rnx.epoch().collect::<Vec<Epoch>>(),
            expected,
            "parsed wrong epoch content"
        );

        let record = rnx.record.as_obs().unwrap();
        for (k, v) in record.iter() {
            assert!(k.flag.is_ok(), "bad epoch flag @{:?}", k.epoch);
            assert!(v.clock_offset.is_none(), "bad clock offset @{:?}", k.epoch);
            for (k, obs_data) in v.observations.iter() {
                // TODO: add more tests
            }
        }
    }
    #[cfg(feature = "flate2")]
    #[cfg(feature = "qc")]
    #[test]
    fn v3_esbc00dnk_r_2020() {
        let rnx =
            Rinex::from_file("../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
                .unwrap();

        test_observation_rinex(
            &rnx,
            "3.05",
            Some("MIXED"),
            "BDS, GAL, GLO, QZSS, GPS, EGNOS, SDCM, BDSBAS",
            "C05, C07, C10, C12, C19, C20, C23, C32, C34, C37,
             E01, E03, E05, E09, E13, E15, E24, E31,
             G02, G05, G07, G08, G09, G13, G15, G18, G21, G27, G28, G30,
             R01, R02, R08, R09, R10, R11, R12, R17, R18, R19,
             S23, S25, S36",
            "C2I, C6I, C7I, D2I, D6I, D7I, L2I, L6I, L7I, S2I, S6I, S7I,
              C1C, C5Q, C6C, C7Q, C8Q, D1C, D5Q, D6C, D7Q, D8Q, L1C, L5Q, L6C,
              L7Q, L8Q, S1C, S5Q, S7Q, S8Q,
              C1C, C1W, C2L, C2W, C5Q, D1C, D2L, D2W, D5Q, L1C, L2L, L2W, L5Q,
              S1C, S1W, S2L, S2W, S5Q,
              C1C, C2L, C5Q, D1C, D2L, D5Q, L1C, L2L, L5Q, S1C, S2L, S5Q,
              C1C, C1P, C2C, C2P, C3Q, D1C, D1P, D2C, D2P, D3Q, L1C, L1P, L2C,
              L2P, L3Q, S1C, S1P, S2C, S2P, S3Q,
              C1C, C5I, D1C, D5I, L1C, L5I, S1C, S5I",
            Some("2020-06-25T00:00:00 GPST"),
            Some("2020-06-25T23:59:30 GPST"),
            evenly_spaced_time_frame!(
                "2020-06-25T00:00:00 GPST",
                "2020-06-25T23:59:30 GPST",
                "30 s"
            ),
        );

        /*
         * Header tb
         */
        let header = rnx.header.clone();

        assert!(
            header.geodetic_marker.is_some(),
            "failed to parse geodetic marker"
        );
        let marker = header.geodetic_marker.unwrap();
        assert_eq!(marker.name, "ESBC00DNK");
        assert_eq!(marker.number(), Some("10118M001".to_string()));
        assert_eq!(marker.marker_type, Some(MarkerType::Geodetic));

        /*
         * Observation specific
         */
        let obs = header.obs.as_ref();
        assert!(obs.is_some());
        let obs = obs.unwrap();

        for (k, v) in &obs.codes {
            if *k == Constellation::GPS {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> =
                    "C1C C1W C2L C2W C5Q D1C D2L D2W D5Q L1C L2L L2W L5Q S1C S1W S2L S2W S5Q"
                        .split_ascii_whitespace()
                        .map(|k| Observable::from_str(k).unwrap())
                        .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else if *k == Constellation::Glonass {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> =
                    "C1C C1P C2C C2P C3Q D1C D1P D2C D2P D3Q L1C L1P L2C L2P L3Q S1C S1P S2C S2P S3Q"
                    .split_ascii_whitespace()
                    .map(|k| Observable::from_str(k).unwrap())
                    .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else if *k == Constellation::BeiDou {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> =
                    "C2I C6I C7I D2I D6I D7I L2I L6I L7I S2I S6I S7I"
                        .split_ascii_whitespace()
                        .map(|k| Observable::from_str(k).unwrap())
                        .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else if *k == Constellation::QZSS {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> =
                    "C1C C2L C5Q D1C D2L D5Q L1C L2L L5Q S1C S2L S5Q"
                        .split_ascii_whitespace()
                        .map(|k| Observable::from_str(k).unwrap())
                        .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else if *k == Constellation::Galileo {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> =
                    "C1C C5Q C6C C7Q C8Q D1C D5Q D6C D7Q D8Q L1C L5Q L6C L7Q L8Q S1C S5Q S6C S7Q S8Q"
                    .split_ascii_whitespace()
                    .map(|k| Observable::from_str(k).unwrap())
                    .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else if *k == Constellation::SBAS {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> = "C1C C5I D1C D5I L1C L5I S1C S5I"
                    .split_ascii_whitespace()
                    .map(|k| Observable::from_str(k).unwrap())
                    .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else {
                panic!("decoded unexpected constellation");
            }
        }

        assert_eq!(header.glo_channels.len(), 23);
        let mut keys: Vec<SV> = header.glo_channels.keys().copied().collect();
        keys.sort();
        assert_eq!(
            vec![
                SV::from_str("R01").unwrap(),
                SV::from_str("R02").unwrap(),
                SV::from_str("R03").unwrap(),
                SV::from_str("R04").unwrap(),
                SV::from_str("R05").unwrap(),
                SV::from_str("R06").unwrap(),
                SV::from_str("R07").unwrap(),
                SV::from_str("R08").unwrap(),
                SV::from_str("R09").unwrap(),
                SV::from_str("R10").unwrap(),
                SV::from_str("R11").unwrap(),
                SV::from_str("R12").unwrap(),
                SV::from_str("R13").unwrap(),
                SV::from_str("R14").unwrap(),
                SV::from_str("R15").unwrap(),
                SV::from_str("R16").unwrap(),
                SV::from_str("R17").unwrap(),
                SV::from_str("R18").unwrap(),
                SV::from_str("R19").unwrap(),
                SV::from_str("R20").unwrap(),
                SV::from_str("R21").unwrap(),
                SV::from_str("R23").unwrap(),
                SV::from_str("R24").unwrap(),
            ],
            keys
        );
        let mut values: Vec<i8> = header.glo_channels.values().copied().collect();
        values.sort();
        assert_eq!(
            vec![
                -7_i8, -7_i8, -4_i8, -4_i8, -3_i8, -2_i8, -2_i8, -1_i8, -1_i8, 0_i8, 0_i8, 1_i8,
                1_i8, 2_i8, 2_i8, 3_i8, 3_i8, 4_i8, 4_i8, 5_i8, 5_i8, 6_i8, 6_i8
            ],
            values
        );

        let record = rnx.record.as_obs().unwrap();
        for (k, v) in record.iter() {
            assert!(k.flag.is_ok(), "bad epoch flag @{:?}", k.epoch);
            assert!(v.clock_offset.is_none(), "bad clock offset @{:?}", k.epoch);
            for (k, obs_data) in v.observations.iter() {
                // TODO add more tests
            }
        }
    }
    #[cfg(feature = "flate2")]
    #[test]
    fn v3_mojn00dnk_r_2020() {
        let rnx =
            Rinex::from_file("../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz")
                .unwrap();

        let header = &rnx.header;
        let obs_header = header.obs.as_ref().unwrap();
        assert_eq!(header.version, Version { major: 3, minor: 5 });
        assert_eq!(header.constellation, Some(Constellation::Mixed));

        let constellations = rnx.constellation().sorted().collect::<Vec<_>>();
        assert_eq!(
            constellations,
            ["GPS", "GLO", "GAL", "BDS", "QZSS", "IRNSS", "EGNOS", "SDCM", "GAGAN", "BDSBAS"]
                .iter()
                .map(|c| Constellation::from_str(c).unwrap())
                .sorted()
                .collect::<Vec<_>>()
        );

        let sv = rnx
            .sv()
            .filter(|sv| sv.constellation == Constellation::QZSS)
            .sorted()
            .collect::<Vec<_>>();
        assert_eq!(
            sv,
            "J01, J02, J03"
                .split(',')
                .map(|j| SV::from_str(j.trim()).unwrap())
                .sorted()
                .collect::<Vec<_>>()
        );

        let sv = rnx
            .sv()
            .filter(|sv| sv.constellation == Constellation::IRNSS)
            .sorted()
            .collect::<Vec<_>>();
        assert_eq!(
            sv,
            "I01, I02, I04, I05, I06, I09"
                .split(',')
                .map(|j| SV::from_str(j.trim()).unwrap())
                .sorted()
                .collect::<Vec<_>>()
        );

        let sv = rnx
            .sv()
            .filter(|sv| sv.constellation.is_sbas())
            .sorted()
            .collect::<Vec<_>>();
        assert_eq!(
            sv,
            "S23, S25, S26, S27, S36, S44"
                .split(',')
                .map(|j| SV::from_str(j.trim()).unwrap())
                .sorted()
                .collect::<Vec<_>>()
        );

        assert_eq!(
            obs_header.time_of_first_obs,
            Some(Epoch::from_str("2020-06-25T00:00:00 GPST").unwrap())
        );
        assert_eq!(
            obs_header.time_of_last_obs,
            Some(Epoch::from_str("2020-06-25T23:59:30 GPST").unwrap())
        );
    }
}
