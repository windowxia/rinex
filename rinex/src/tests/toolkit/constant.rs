use crate::meteo::Record as MetRecord;
use crate::observation::Record as ObsRecord;
use crate::Rinex;

/*
 * Test: panic if given RINEX content is not equal to given constant
 */
pub fn is_constant_rinex(rnx: &Rinex, constant: f64, tolerance: f64) {
    if let Some(record) = rnx.record.as_obs() {
        is_constant_obs_record(record, constant, tolerance)
    } else if let Some(record) = rnx.record.as_meteo() {
        is_constant_meteo_record(record, constant, tolerance)
    } else {
        unimplemented!("is_constant_rinex({})", rnx.header.rinex_type);
    }
}

pub fn is_null_rinex(rnx: &Rinex, tolerance: f64) {
    is_constant_rinex(rnx, 0.0_f64, tolerance)
}

fn is_constant_obs_record(record: &ObsRecord, constant: f64, tolerance: f64) {
    for (k, v) in record.iter() {
        let epoch = k.epoch;
        if let Some(clk) = v.clock_offset {
            let err = (clk - constant).abs();
            if err > tolerance {
                panic!("@{:?} rcvr clock error {} != {}", epoch, clk, constant);
            }
        }
        for (k, v) in v.observations.iter() {
            let err = (v.value - constant).abs();
            if err > tolerance {
                panic!("@{:?} data error {} != {}", epoch, v.value, constant);
            }
        }
    }
}

fn is_constant_meteo_record(record: &MetRecord, constant: f64, tolerance: f64) {
    for (t, observations) in record.iter() {
        for (observable, value) in observations {
            let err = (value - constant).abs();
            if err > tolerance {
                panic!("@{:?} data error {} != {}", t, value, constant);
            }
        }
    }
}
