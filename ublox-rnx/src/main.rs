use log::{debug, error, info, trace, warn};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::str::FromStr;
use thiserror::Error;

use rinex::navigation::{IonMessage, KbModel, KbRegionCode};

use rinex::{
    carrier::Carrier,
    observation::{LliFlags, ObservationData},
    prelude::{Duration, Epoch, EpochFlag, Header, Observable, Rinex},
    record::Record,
};

use gnss_rs::prelude::{Constellation, SV};

extern crate ublox;

use ublox::{
    CfgMsgAllPorts, CfgMsgAllPortsBuilder, CfgPrtUart, CfgPrtUartBuilder, DataBits, GpsFix,
    InProtoMask, NavSat, NavStatusFlags, NavStatusFlags2, NavTimeUtcFlags, OutProtoMask, PacketRef,
    Parity, RecStatFlags, RxmRawx, RxmRawxInfo, StopBits, TrkStatFlags, UartMode, UartPortId,
};

mod cli;
use cli::Cli;

mod device;

/// output stream
pub enum Output {
    /// Print on stdout
    StdOut,
}

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("non supported constellation #{0}")]
    NonSupportedConstellation(u8),
    #[error("unknown carrier signal")]
    UnknownSignal,
    #[error("unknown gps signal {0}")]
    UnknownGpsSignal(u8),
    #[error("unknown galileo signal {0}")]
    UnknownGalileoSignal(u8),
    #[error("unknown beidou signal {0}")]
    UnknownBeiDouSignal(u8),
    #[error("unknown qzss signal {0}")]
    UnknownQzssSignal(u8),
    #[error("unknown lnav signal {0}")]
    UnknownLnavSignal(u8),
    #[error("unknown glonass signal {0}")]
    UnknownGlonassSignal(u8),
}

fn ubx2constell(id: u8) -> Result<Constellation, Error> {
    match id {
        0 => Ok(Constellation::GPS),
        1 => Ok(Constellation::Galileo),
        2 => Ok(Constellation::Glonass),
        3 => Ok(Constellation::BeiDou),
        _ => Err(Error::NonSupportedConstellation(id)),
    }
}

fn ubx2sv(gnss_id: u8, sv_id: u8) -> Result<SV, Error> {
    Ok(SV {
        constellation: ubx2constell(gnss_id)?,
        prn: sv_id,
    })
}

fn ubx2gpscarrier(freq_id: u8) -> Result<Carrier, Error> {
    match freq_id {
        1 => Ok(Carrier::L1),
        2 => Ok(Carrier::L2),
        5 => Ok(Carrier::L5),
        _ => Err(Error::UnknownGpsSignal(freq_id)),
    }
}

fn ubx2glocarrier(freq_id: u8) -> Result<Carrier, Error> {
    Err(Error::UnknownGlonassSignal(freq_id))
}

fn ubx2galcarrier(freq_id: u8) -> Result<Carrier, Error> {
    Err(Error::UnknownGalileoSignal(freq_id))
}

fn ubx2bdscarrier(freq_id: u8) -> Result<Carrier, Error> {
    Err(Error::UnknownBeiDouSignal(freq_id))
}

fn ubx2qzsscarrier(freq_id: u8) -> Result<Carrier, Error> {
    Err(Error::UnknownQzssSignal(freq_id))
}

fn ubx2lnavcarrier(freq_id: u8) -> Result<Carrier, Error> {
    Err(Error::UnknownLnavSignal(freq_id))
}

fn ubx2carrier(gnss_id: u8, freq_id: u8) -> Result<Carrier, Error> {
    let gnss = ubx2constell(gnss_id)?;
    match gnss {
        Constellation::GPS => ubx2gpscarrier(freq_id),
        Constellation::Galileo => ubx2galcarrier(freq_id),
        Constellation::BeiDou => ubx2bdscarrier(freq_id),
        Constellation::Glonass => ubx2glocarrier(freq_id),
        Constellation::IRNSS => ubx2lnavcarrier(freq_id),
        Constellation::QZSS => ubx2qzsscarrier(freq_id),
        _ => Err(Error::UnknownSignal),
    }
}

fn obsrinex(cli: &Cli) -> Rinex {
    let mut obs = Rinex {
        header: {
            Header::basic_obs().with_comment(&format!(
                "ubx2rnx v{}    https://github.com/georust/rinex",
                env!("CARGO_PKG_VERSION")
            ))
        },
        comments: BTreeMap::new(),
        record: Record::default(),
    };
    if cli.crinex() {
        obs.crnx2rnx();
    }
    obs
}

fn output_setup(cli: &Cli) -> Result<Box<dyn Write>, Box<dyn std::error::Error>> {
    match cli.output() {
        Output::StdOut => Ok(Box::new(std::io::stdout().lock())),
        // Output::File(path) => Ok(Box::new(std::fs::File::create(&path)?)),
        // Output::Ftp(url) => Ok(Box::new(ftp_client::FtpClient::new())),
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // cli
    let cli = Cli::new();

    // Device configuration
    let port = cli.port();
    let baud_rate = match cli.baudrate() {
        Ok(b) => b,
        Err(e) => {
            error!("failed to parse baud_rate: {}", e);
            9600
        },
    };

    info!("connecting to {}, baud: {}", port, baud_rate);

    // open device
    let port = serialport::new(port.clone(), baud_rate)
        .open()
        .unwrap_or_else(|_| panic!("failed to open serial port \"{}\"", port));
    let mut device = device::Device::new(port);

    // Enable UBX protocol on all ports
    device.write_all(
        &CfgPrtUartBuilder {
            portid: UartPortId::Uart1,
            reserved0: 0,
            tx_ready: 0,
            mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
            baud_rate,
            in_proto_mask: InProtoMask::all(),
            out_proto_mask: OutProtoMask::UBLOX,
            flags: 0,
            reserved5: 0,
        }
        .into_packet_bytes(),
    )?;
    device.wait_for_ack::<CfgPrtUart>().unwrap();

    /*
     * Header
     * (one time) configuration
     */
    //TODO
    //CfgNav5 : model, dynamics..
    //CfgNav5X : min_svs, aiding, wkn, ppp..
    //AidIni

    /*
     * Observation frames
     */
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<RxmRawx>([0, 1, 0, 0, 0, 0]).into_packet_bytes(),
        )
        .unwrap();
    device.wait_for_ack::<CfgMsgAllPorts>().unwrap();

    /*
     * Navigation Frames
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<NavSat>([0, 1, 0, 0, 0, 0]).into_packet_bytes(),
        )
        .unwrap();
    device.wait_for_ack::<CfgMsgAllPorts>().unwrap();
     */

    // Create basic structure with customized headers
    //  that we will generate right away,
    //  afterwards
    let obsrinex = obsrinex(&cli);

    let mut obs_header = Header::basic_obs();
    let mut obs_epoch = Epoch::default();
    let mut obs_flag = EpochFlag::default();
    let mut obs_clk_offset = Option::<f64>::None;
    let mut observations = BTreeMap::<SV, HashMap<Observable, ObservationData>>::new();

    let mut publish = false;
    let mut content: String = String::new();
    let mut epoch = Epoch::default();

    // observation
    let mut _observable = Observable::default();
    let mut lli: Option<LliFlags> = None;
    let mut obs_data = ObservationData::default();

    let mut uptime = Duration::default();

    let mut fix_type = GpsFix::NoFix; // current fix status
    let mut fix_flags = NavStatusFlags::empty(); // current fix flag
    let mut nav_status = NavStatusFlags2::Inactive;

    /*
     * Open output interface
     */
    let mut output = output_setup(&cli)?;

    /*
     * Initialize a header
     */
    output.write(obsrinex)?;

    // main loop
    loop {
        let _ = device.update(|packet| {
            match packet {
                /*
                 * Configuration frames:
                 * should be depiceted by HEADER section
                 */
                //PacketRef::CfgRate(pkt) => {
                //    //TODO EPOCH INTERVAL
                //    let gps_rate = pkt.measure_rate_ms();
                //    //TODO EPOCH INTERVAL
                //    let nav_rate = pkt.nav_rate();
                //    //TODO reference time
                //    let time = pkt.time_ref();
                //},
                PacketRef::CfgNav5(pkt) => {
                    // Dynamic model
                    let _dyn_model = pkt.dyn_model();
                },
                /*
                 * Mon frames
                 */
                PacketRef::MonHw(_pkt) => {
                    //let jamming = pkt.jam_ind(); //TODO
                    //antenna problem:
                    // pkt.a_status();
                    // pkt.a_power();
                },
                PacketRef::MonGnss(_pkt) => {
                    //pkt.supported(); // GNSS
                    //pkt.default(); // GNSS
                    //pkt.enabled(); //GNSS
                },
                PacketRef::MonVer(pkt) => {
                    //UBX revision
                    pkt.software_version();
                    pkt.hardware_version();
                },
                /*
                 * RAW frames
                 */
                PacketRef::RxmRawx(pkt) => {
                    let rcv_tow = pkt.rcv_tow();
                    // let leap_s = pkt.leap_s();
                    if pkt.rec_stat().intersects(RecStatFlags::CLK_RESET) {
                        // notify reset event
                        if let Some(ref mut lli) = lli {
                            *lli |= LliFlags::LOCK_LOSS;
                        } else {
                            lli = Some(LliFlags::LOCK_LOSS);
                        }
                        obs_flag = EpochFlag::CycleSlip;
                    }
                    obs_data.lli = lli;

                    for meas in pkt.measurements() {
                        let sv_id = meas.sv_id();
                        let gnss_id = meas.gnss_id();

                        if let Ok(sv) = ubx2sv(gnss_id, sv_id) {
                            let do_mes = meas.do_mes();
                            let cno = meas.cno();
                            let freq_id = meas.freq_id();
                            if let Ok(carrier) = ubx2carrier(gnss_id, freq_id) {
                                let trk_stat = meas.trk_stat();
                                if trk_stat.intersects(TrkStatFlags::PR_VALID) {
                                    let pr = meas.pr_mes();
                                }
                                if trk_stat.intersects(TrkStatFlags::CP_VALID) {
                                    let cp = meas.cp_mes();
                                }
                                if trk_stat.intersects(TrkStatFlags::HALF_CYCLE) {}
                                if trk_stat.intersects(TrkStatFlags::SUB_HALF_CYCLE) {}
                            } else {
                                error!(
                                    "{:?}: failed to identify carrier signal {}",
                                    obs_epoch, freq_id
                                );
                            }
                        } else {
                            error!(
                                "{:?}: failed to identify sat_vehicle {}/{}",
                                obs_epoch, gnss_id, sv_id
                            );
                        }
                    }
                },
                /*
                 * NAVIGATION
                 */
                //PacketRef::NavSat(pkt) => {
                //    for sv in pkt.svs() {
                //        let gnss = constell_from_ubx(sv.gnss_id());
                //        if gnss.is_ok() {
                //            let _elev = sv.elev();
                //            let _azim = sv.azim();
                //            let _pr_res = sv.pr_res();
                //            let _flags = sv.flags();

                //            let _sv = SV {
                //                constellation: gnss.unwrap(),
                //                prn: sv.sv_id(),
                //            };

                //            // flags.sv_used()
                //            //flags.health();
                //            //flags.quality_ind();
                //            //flags.differential_correction_available();
                //            //flags.ephemeris_available();
                //        }
                //    }
                //},
                //PacketRef::NavTimeUTC(pkt) => {
                //    if pkt.valid().intersects(NavTimeUtcFlags::VALID_UTC) {
                //        // leap seconds already known
                //        let e = Epoch::maybe_from_gregorian(
                //            pkt.year().into(),
                //            pkt.month(),
                //            pkt.day(),
                //            pkt.hour(),
                //            pkt.min(),
                //            pkt.sec(),
                //            pkt.nanos() as u32,
                //            TimeScale::UTC,
                //        );
                //        if e.is_ok() {
                //            epoch = e.unwrap();
                //        }
                //    }
                //},
                //PacketRef::NavStatus(pkt) => {
                //    itow = pkt.itow();
                //    fix_type = pkt.fix_type();
                //    fix_flags = pkt.flags();
                //    nav_status = pkt.flags2();
                //    uptime = Duration::from_milliseconds(pkt.uptime_ms() as f64);
                //    trace!("uptime: {}", uptime);
                //},
                PacketRef::NavEoe(pkt) => {
                    // itow = pkt.itow();
                    // reset Epoch
                    // lli = None;
                    // epoch_flag = EpochFlag::default();
                },
                /*
                 * OBSERVATION
                 */
                PacketRef::NavClock(pkt) => {
                    let _bias = pkt.clk_b();
                    let _drift = pkt.clk_d();
                    // pkt.t_acc(); // phase accuracy
                    // pkt.f_acc(); // frequency accuracy
                },
                /*
                 * Errors, Warnings
                 */
                PacketRef::InfTest(pkt) => {
                    if let Some(msg) = pkt.message() {
                        trace!("{}", msg);
                    }
                },
                PacketRef::InfDebug(pkt) => {
                    if let Some(msg) = pkt.message() {
                        debug!("{}", msg);
                    }
                },
                PacketRef::InfNotice(pkt) => {
                    if let Some(msg) = pkt.message() {
                        info!("{}", msg);
                    }
                },
                PacketRef::InfError(pkt) => {
                    if let Some(msg) = pkt.message() {
                        error!("{}", msg);
                    }
                },
                PacketRef::InfWarning(pkt) => {
                    if let Some(msg) = pkt.message() {
                        warn!("{}", msg);
                    }
                },
                _ => {},
            }
        });

        if publish {
            //if output.write(&content).is_err() {
            //    warn!("{:?}: failed to generate new observations", t_obs);
            //}
        }

        content.clear();
    }
    //Ok(())
}
