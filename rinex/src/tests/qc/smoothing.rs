#[cfg(test)]
mod test {
    use crate::prelude::*;
    use crate::preprocessing::*;
    use std::str::FromStr;
    #[test]
    #[ignore]
    fn v3_duth0630_hatch_filter() {
        //    let initial_rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
        //    let filter = Filter::from_str("smooth:hatch:c1c,c2p,l1c,l2p").unwrap();
        //    let filtered = initial_rinex.filter(filter);

        //    let expected: Vec<(&str, &str, Vec<f64>)> = vec![(
        //        "G01",
        //        "C1C",
        //        vec![
        //            20243517.560,
        //            1.0 / 2.0 * 20805393.080
        //                + (20243517.560 + (109333085.615 - 106380411.418)) * (2.0 - 1.0) / 2.0,
        //            1.0 / 3.0 * 21653418.260
        //                + ((1.0 / 2.0 * 20805393.080
        //                    + (20243517.560 + (109333085.615 - 106380411.418)) * (2.0 - 1.0) / 2.0)
        //                    + (113789485.670 - 109333085.615))
        //                    * (3.0 - 1.0)
        //                    / 3.0,
        //        ],
        //    )];
        //    testbench("hatch", expected, &filtered);

        //    let expected: Vec<(&str, &str, Vec<f64>)> = vec![(
        //        "R10",
        //        "C2P",
        //        vec![
        //            23044984.180,
        //            1.0 / 2.0 * 22432243.520
        //                + (23044984.180 + (122842738.811 - 106380411.418)) * (2.0 - 1.0) / 2.0,
        //            1.0 / 3.0 * 22235350.560
        //                + ((1.0 / 2.0 * 22432243.520
        //                    + (23044984.180 + (122842738.811 - 106380411.418)) * (2.0 - 1.0) / 2.0)
        //                    + (118526944.203 - 119576492.916))
        //                    * (3.0 - 1.0)
        //                    / 3.0,
        //        ],
        //    )];
        //    testbench("hatch", expected, &filtered);
    }
}
