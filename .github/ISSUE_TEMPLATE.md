## GeoRust RINEX Issue Template

Thank you for using our toolbox and contributing to it.  

Before opening an issue, make sure your tools are [up to date](https://github.com/georust/rinex/releases/latest).

[Follow this guideline](#application-bug-report) if you want to report a bug or are experience troubles running one of our applications.
[Use this guideline](#library-bug-report) if you're a developer and are facing issues using one of our libraries.

## Application bug report

Copy and paste the command line so we can reproduce it on our side.   
Use the `RUST_LOG` (environment variable) to activate the logger and provide its output,
either in the bug report directly or as attached file (use compression at your convenience).

Example:

```bash
RUST_LOG=trace ./target/release/rinex-cli \
    -f test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    -f test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz \
    -f test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz  \
    -f test_resources/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz \
    -P GPS -p | tee logs.txt

[2024-04-27T13:14:18Z TRACE rinex_cli::preprocessing] applied filter "GPS"
[2024-04-27T13:14:18Z DEBUG rinex_cli] Primary: "ESBC00DNK_R_20201770000_01D_30S_MO"
    Observation: ["test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz"]
    Broadcast Navigation: ["test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz"]
    High Precision Orbit (SP3): ["test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz"]
    High Precision Clock: ["test_resources/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz"]
[2024-04-27T13:14:18Z INFO  rinex_cli] session workspace is "WORKSPACE/ESBC00DNK_R_20201770000_01D_30S_MO"
[2024-04-27T13:14:18Z INFO  rinex_cli] position defined in dataset: (3582105.291, 532589.7313, 5232754.8054) [ECEF] (lat=55.49356°, lon=8.45682°
[2024-04-27T13:14:18Z INFO  rinex_cli::positioning] Using CodePPP default preset: Config {}
[...]
```

## Library bug report

Copy and paste a minimal reproducible example, so we can easily reproduce the problem on our side.


Example:

```rust
use rinex::prelude::*;
let rinex = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
    .unwrap();
```
