#[cfg(test)]
mod test {
    use crate::prelude::*;
    use itertools::Itertools;
    use std::str::FromStr;
    #[test]
    fn sv_mask_filter() {
        for (fp, op, sv_list, num_sat, c) in [(
            "OBS/V3/DUTH0630.22O",
            MaskOperand::Equals,
            "G01,G03",
            2,
            vec![Constellation::GPS],
        )] {
            let mut rinex = Rinex::from_file(&format!(
                "{}/../test_resources/{}",
                env!("CARGO_MANIFEST_DIR"),
                fp
            ))
            .unwrap();
            let mask = MaskFilter {
                operand: op,
                token: MaskToken::SV(
                    sv_list
                        .split(",")
                        .map(|c| SV::from_str(c.trim()).unwrap())
                        .collect(),
                ),
            };
            rinex.mask_mut(&mask);

            let sv = rinex.sv().collect::<Vec<_>>();
            assert_eq!(sv.len(), num_sat);

            let constells = rinex.constellation().sorted().collect::<Vec<_>>();
            assert_eq!(constells, c);
        }
    }
    //#[test]
    //#[ignore]
    //fn v2_cari0010_07m_phys_filter() {
    //    let rnx = Rinex::from_file("../test_resources/MET/V2/cari0010.07m").unwrap();
    //    let dut = rnx.filter(filter!("L1C"));
    //    assert_eq!(dut.observable().count(), 0);
    //    let _dut = rnx.filter(filter!("TD"));
    //    assert_eq!(rnx.observable().count(), 1);
    //}
    //#[test]
    //fn v2_clar0020_00m_phys_filter() {
    //    let rnx = Rinex::from_file("../test_resources/MET/V2/clar0020.00m").unwrap();
    //    let dut = rnx.filter(filter!("L1C"));
    //    assert_eq!(dut.observable().count(), 0);
    //    let dut = rnx.filter(filter!("PR"));
    //    assert_eq!(dut.observable().count(), 1);
    //}
    //#[test]
    //fn v2_cari0010_07m_time_filter() {
    //    let rnx = Rinex::from_file("../test_resources/MET/V2/cari0010.07m").unwrap();
    //    let dut = rnx.filter(filter!(">2000-01-02T22:00:00 UTC"));
    //    assert_eq!(dut.epoch().count(), 0);
    //    let dut = rnx.filter(filter!("<=1996-04-02T00:00:00 UTC"));
    //    assert_eq!(dut.epoch().count(), 3);
    //    let dut = rnx.filter(filter!("<=1996-04-01T00:00:30 UTC"));
    //    assert_eq!(dut.epoch().count(), 2);
    //    let dut = rnx.filter(filter!("< 1996-04-01T00:00:30 UTC"));
    //    assert_eq!(dut.epoch().count(), 1);
    //}
}
