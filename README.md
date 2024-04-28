RINEX 
=====

[![Rust](https://github.com/georust/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)

[![minimum rustc: 1.64](https://img.shields.io/badge/minimum%20rustc-1.64-blue?logo=rust)](https://www.whatrustisit.com)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/georust/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/georust/rinex/blob/main/LICENSE-MIT) 

Rust tool suites to parse, analyze and process [RINEX Data](https://en.wikipedia.org/wiki/RINEX).

The [Wiki pages](https://github.com/georust/rinex/wiki) are the main documentation. They contain
several examples spanning different GNSS applications.

Use the [Github Issues](https://github.com/georust/rinex/issues) to report bugs by following our quick procedure.  
Open a [Discussion](https://github.com/georust/rinex/discussions) or reach out to us on [Discord](https://discord.gg/Fp2aape) 
for any questions.

## Advantages :rocket: 

- Fast data browsing
- Fast SPP/PPP solution solver
- Open sources
- Seamless yet efficient Hatanaka and Gzip compression
- RINEX V4 fully supported
- All RINEX format supported
- DORIS, IONEX and ANTEX support
- SP3 (high precision orbits) fully supported
- Several pre processing algorithms:
  - [File merging](https://github.com/georust/rinex/wiki/file-merging)
  - [Time binning](https://github.com/georust/rinex/wiki/time-binning)
  - [Filtering](https://github.com/georust/rinex/wiki/Preprocessing)
- Several post processing operations
  - [Position solver](https://github.com/georust/rinex/wiki/Positioning)
  - [Graphical analysis](https://github.com/georust/rinex/wiki/Graph-Mode)
  - [CGGTTS solutions solver](https://github.com/georust/rinex/wiki/CGGTTS)
- All modern GNSS constellations, codes and signals
- Time scales: GPST, BDT, GST, UTC
- [SBAS support](https://docs.rs/gnss-rs/2.1.3/gnss_rs/constellation/enum.Constellation.html)
- High precision phase data (micro cycle precision) theoretically supported but not tested yet
- [Quality Check (QC)](https://github.com/georust/rinex/wiki/Quality-Check): file quality and statistical analysis to help precise positioning
(historical `teqc` function).

## Disadvantages :warning:

- QZNSST is represented as GPST at the moment.
- GLONASST, IRNSST are not supported yet
- SPP/PPP solving only feasible with GPS and Galileo at the moment
- We cannot process proprietary formats like BINEX
- Some data production features might be missing as we're currently focused
on data processing

## Repository 

* [`rinex`](rinex/) is the core library 
* [`rinex-cli`](rinex-cli/) : RINEX post processing
and broader GNSS processing by command line (no GUI available yet).
It's growing as some sort of Anubis + glab + Rtklib combination.
It also supports some operations that are similar to `teqc`.
It can generate PVT and CGGTTS solutions, QC reports or can be used as a data extractor
to third party tools.
The application is auto-generated for a few architectures, download it from the 
[release portal](https://github.com/georust/rinex/releases)

* [`sp3`](sp3/) High Precision Orbits (by IGS) 
* [`rinex-qc`](rinex-qc/) is a library dedicated to RINEX and GNSS data analysis and processing.
* [`qc-traits`](qc-traits/) is where analysis and processing Traits are declared
* [`sinex`](sinex/) SINEX dedicated core library, is work in progress and not integrated
to the processing toolbox yet.
* [`ublox-rnx`](ublox-rnx/) is an application intended to generate RINEX files
from a uBlox receiver. This application is work in progress at the moment.

## Other tools and relevant Ecosystems

* [Nyx-space](https://github.com/nyx-space/nyx): Navigation, Orbital attitude
* [Hifitime](https://github.com/nyx-space/hifitime): Precise Timing, Timescales, ...
* [CGGTTS](https://github.com/gwbres/cggtts): Common View Time Transfer
- [RTK-RS](https://github.com/rtk-rs/gnss-rtk): Precise Positioning
* [GNSS definitions in Rust](https://github.com/rtk-rs/gnss): GNSS library

## Citation and referencing

If you need to reference this work, please use the following model:

`GeoRust RINEX Team (2023), RINEX: analysis and processing (Apache-2/MIT), https://georust.org`

Formats & revisions
===================

The core library supports parsing RINEX V4.00 and the current behavior is to fail
on higher revisions. NAV V4 is correctly supported as described in the following table.

We support the latest revisions for both IONEX and Clock RINEX.

We support the latest (rev D) SP3 format.

RINEX formats & applications
============================

| Type                       | Parser            | Writer              |  CLI                 |      Content         | Record Iteration     | Timescale  |
|----------------------------|-------------------|---------------------|----------------------|----------------------|----------------------| -----------|
| Navigation  (NAV)          | :heavy_check_mark:| :construction:      |  :heavy_check_mark: :chart_with_upwards_trend:  | Ephemerides, Ionosphere models | Epoch | SV System time broadcasting this message |
| Observation (OBS)          | :heavy_check_mark:| :heavy_check_mark: | :heavy_check_mark:  :chart_with_upwards_trend: | Phase, Pseudo Range, Doppler, SSI | Epoch | GNSS |
|  CRINEX  (Compressed OBS)  | :heavy_check_mark:| RNX2CRX1 :heavy_check_mark: RNX2CRX3 :construction:  | :heavy_check_mark:  :chart_with_upwards_trend:  |  Phase, Pseudo Range, Doppler, SSI | Epoch | GNSS |
|  Meteorological data (MET) | :heavy_check_mark:| :heavy_check_mark:  | :heavy_check_mark: :chart_with_upwards_trend:  | Meteo sensors data (Temperature, Moisture..) | Epoch | UTC | 
|  Clocks (CLK)              | :heavy_check_mark:| :construction:      | :heavy_check_mark: :chart_with_upwards_trend:  | Precise SV and Reference Clock states |  Epoch | UTC |
|  Antenna (ATX)             | :heavy_check_mark:| :construction:      | :construction:   | Precise RX/SV Antenna calibration | `antex::Antenna` | :heavy_minus_sign: |
|  Ionosphere Maps  (IONEX)  | :heavy_check_mark:|  :construction:     | :heavy_check_mark:  :chart_with_upwards_trend: | Ionosphere Electron density | Epoch | UTC |
|  DORIS RINEX               | :heavy_check_mark:|  :construction:     | :construction:       | Phase, Pseudo Range, Temperature and other measurements | Epoch | TAI |
|  SINEX  (SNX)              | :construction:    |  :construction:     | :heavy_minus_sign:   | SINEX are special RINEX, they are managed by a dedicated [core library](sinex/) | Epoch | :question: |
|  Troposphere  (TRO)        | :construction:    |  :construction:     | :question:           | Troposphere modeling | Epoch | :question: |
|  Bias  (BIA)               | :heavy_check_mark: |  :construction:    | :question:           | Bias estimates, like DCB.. | Epoch | :question: |

:heavy_check_mark: means all revisions supported   
:construction: : means Work in Progress   

__CLI__ : possibility to [load this format](https://github.com/georust/rinex/wiki/file-loading) in the apps.  
__CLI__ + :chart_with_upwards_trend: : possibility to [project or extract and plot](https://github.com/georust/rinex/wiki/graph-mode) this format.


Other formats
=============

`RINEX-Cli` accepts more than RINEX data.  

| Type                       | Parser            | Writer              |  CLI                 |      Content         | Record Iteration     | Timescale  |
|----------------------------|-------------------|---------------------|----------------------|----------------------| ---------------------| ---------- |
| SP3                        | :heavy_check_mark:| :construction: Work in progress | :heavy_check_mark: :chart_with_upwards_trend:  | High precision SV orbital state | Epoch | GNSS |

File formats
============

| Format                 | File name restrictions            |    Support                         |
|------------------------|-----------------------------------|------------------------------------|
| RINEX                  | :heavy_minus_sign:                | :heavy_check_mark:                 |
| CRINEX                 | :heavy_minus_sign:                | :heavy_check_mark:                 | 
| gzip compressed RINEX  | Name must end with `.gz`          | `--flate2` feature must be enabled |
| gzip compressed CRINEX | Name must end with `.gz`          | `--flate2` feature must be enabled |
| DORIS RINEX            | :heavy_minus_sign:                | :heavy_check_mark:                 |
| gzip compressed DORIS  | Name must end with `.gz`          | `--flate2` feature must be enabled |
| SP3                    | :heavy_minus_sign:                | :heavy_check_mark:                 | 
| gzip compressed SP3    | Name must end with `.gz`          | `--flate2` feature must be enabled | 
| BINEX                  | :heavy_minus_sign:                | :heavy_minus_sign: We do not support proprietary formats |
| UBX                    | :heavy_minus_sign:                | :construction: Work in progress    |

:heavy_minus_sign: No restrictions: file names do not have to follow naming conventions.  

Special Thanks
==============

These tools would not exist without the great libraries written by C. Rabotin, 
[check out his work](https://github.com/nyx-space).  

Some features would not exist without the invaluable help of J. Lesouple, through
our countless discussions. Check out his 
[PhD manuscript (french)](http://perso.recherche.enac.fr/~julien.lesouple/fr/publication/thesis/THESIS.pdf?fbclid=IwAR3WlHm0eP7ygRzywbL07Ig-JawvsdCEdvz1umJJaRRXVO265J9cp931YyI)

Contributions
=============

[Contribution guidelines and hints](CONTRIBUTING.md) will help you navigate this toolbox quicker.
