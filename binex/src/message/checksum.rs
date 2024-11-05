//! Checksum calculator
use lazy_static::lazy_static;
use md5::{Digest, Md5};

/// Checksum caculator
#[derive(Debug, Copy, Clone)]
pub enum Checksum {
    XOR8,
    XOR16,
    XOR32,
    MD5,
}

lazy_static! {
    static ref CRC16_TABLE: [u16; 256] = [
        0x0000, 0x1021, 0x2042, 0x3063, 0x4084, 0x50A5, 0x60C6, 0x70E7, 0x8108, 0x9129, 0xA14A,
        0xB16B, 0xC18C, 0xD1AD, 0xE1CE, 0xF1EF, 0x1231, 0x0210, 0x3273, 0x2252, 0x52B5, 0x4294,
        0x72F7, 0x62D6, 0x9339, 0x8318, 0xB37B, 0xA35A, 0xD3BD, 0xC39C, 0xF3FF, 0xE3DE, 0x2462,
        0x3443, 0x0420, 0x1401, 0x64E6, 0x74C7, 0x44A4, 0x5485, 0xA56A, 0xB54B, 0x8528, 0x9509,
        0xE5EE, 0xF5CF, 0xC5AC, 0xD58D, 0x3653, 0x2672, 0x1611, 0x0630, 0x76D7, 0x66F6, 0x5695,
        0x46B4, 0xB75B, 0xA77A, 0x9719, 0x8738, 0xF7DF, 0xE7FE, 0xD79D, 0xC7BC, 0x48C4, 0x58E5,
        0x6886, 0x78A7, 0x0840, 0x1861, 0x2802, 0x3823, 0xC9CC, 0xD9ED, 0xE98E, 0xF9AF, 0x8948,
        0x9969, 0xA90A, 0xB92B, 0x5AF5, 0x4AD4, 0x7AB7, 0x6A96, 0x1A71, 0x0A50, 0x3A33, 0x2A12,
        0xDBFD, 0xCBDC, 0xFBBF, 0xEB9E, 0x9B79, 0x8B58, 0xBB3B, 0xAB1A, 0x6CA6, 0x7C87, 0x4CE4,
        0x5CC5, 0x2C22, 0x3C03, 0x0C60, 0x1C41, 0xEDAE, 0xFD8F, 0xCDEC, 0xDDCD, 0xAD2A, 0xBD0B,
        0x8D68, 0x9D49, 0x7E97, 0x6EB6, 0x5ED5, 0x4EF4, 0x3E13, 0x2E32, 0x1E51, 0x0E70, 0xFF9F,
        0xEFBE, 0xDFDD, 0xCFFC, 0xBF1B, 0xAF3A, 0x9F59, 0x8F78, 0x9188, 0x81A9, 0xB1CA, 0xA1EB,
        0xD10C, 0xC12D, 0xF14E, 0xE16F, 0x1080, 0x00A1, 0x30C2, 0x20E3, 0x5004, 0x4025, 0x7046,
        0x6067, 0x83B9, 0x9398, 0xA3FB, 0xB3DA, 0xC33D, 0xD31C, 0xE37F, 0xF35E, 0x02B1, 0x1290,
        0x22F3, 0x32D2, 0x4235, 0x5214, 0x6277, 0x7256, 0xB5EA, 0xA5CB, 0x95A8, 0x8589, 0xF56E,
        0xE54F, 0xD52C, 0xC50D, 0x34E2, 0x24C3, 0x14A0, 0x0481, 0x7466, 0x6447, 0x5424, 0x4405,
        0xA7DB, 0xB7FA, 0x8799, 0x97B8, 0xE75F, 0xF77E, 0xC71D, 0xD73C, 0x26D3, 0x36F2, 0x0691,
        0x16B0, 0x6657, 0x7676, 0x4615, 0x5634, 0xD94C, 0xC96D, 0xF90E, 0xE92F, 0x99C8, 0x89E9,
        0xB98A, 0xA9AB, 0x5844, 0x4865, 0x7806, 0x6827, 0x18C0, 0x08E1, 0x3882, 0x28A3, 0xCB7D,
        0xDB5C, 0xEB3F, 0xFB1E, 0x8BF9, 0x9BD8, 0xABBB, 0xBB9A, 0x4A75, 0x5A54, 0x6A37, 0x7A16,
        0x0AF1, 0x1AD0, 0x2AB3, 0x3A92, 0xFD2E, 0xED0F, 0xDD6C, 0xCD4D, 0xBDAA, 0xAD8B, 0x9DE8,
        0x8DC9, 0x7C26, 0x6C07, 0x5C64, 0x4C45, 0x3CA2, 0x2C83, 0x1CE0, 0x0CC1, 0xEF1F, 0xFF3E,
        0xCF5D, 0xDF7C, 0xAF9B, 0xBFBA, 0x8FD9, 0x9FF8, 0x6E17, 0x7E36, 0x4E55, 0x5E74, 0x2E93,
        0x3EB2, 0x0ED1, 0x1EF0,
    ];
    static ref CRC32_TABLE: [u32; 256] = [
        0x0000, 0x4C11B7, 0x98236E, 0xD432D9, 0x13046DC, 0x17C576B, 0x1A865B2, 0x1E47405,
        0x2608DB8, 0x22C9C0F, 0x2F8AED6, 0x2B4BF61, 0x350CB64, 0x31CDAD3, 0x3C8E80A, 0x384F9BD,
        0x4C11B70, 0x48D0AC7, 0x459381E, 0x41529A9, 0x5F15DAC, 0x5BD4C1B, 0x5697EC2, 0x5256F75,
        0x6A196C8, 0x6ED877F, 0x639B5A6, 0x675A411, 0x791D014, 0x7DDC1A3, 0x709F37A, 0x745E2CD,
        0x98236E0, 0x9CE2757, 0x91A158E, 0x9560439, 0x8B2703C, 0x8FE618B, 0x82A5352, 0x86642E5,
        0xBE2BB58, 0xBAEAAEF, 0xB7A9836, 0xB368981, 0xAD2FD84, 0xA9EEC33, 0xA4ADEEA, 0xA06CF5D,
        0xD432D90, 0xD0F3C27, 0xDDB0EFE, 0xD971F49, 0xC736B4C, 0xC3F7AFB, 0xCEB4822, 0xCA75995,
        0xF23A028, 0xF6FB19F, 0xFBB8346, 0xFF792F1, 0xE13E6F4, 0xE5FF743, 0xE8BC59A, 0xEC7D42D,
        0x13046DC0, 0x13487C77, 0x139C4EAE, 0x13D05F19, 0x12342B1C, 0x12783AAB, 0x12AC0872,
        0x12E019C5, 0x1164E078, 0x1128F1CF, 0x11FCC316, 0x11B0D2A1, 0x1054A6A4, 0x1018B713,
        0x10CC85CA, 0x1080947D, 0x17C576B0, 0x17896707, 0x175D55DE, 0x17114469, 0x16F5306C,
        0x16B921DB, 0x166D1302, 0x162102B5, 0x15A5FB08, 0x15E9EABF, 0x153DD866, 0x1571C9D1,
        0x1495BDD4, 0x14D9AC63, 0x140D9EBA, 0x14418F0D, 0x1A865B20, 0x1ACA4A97, 0x1A1E784E,
        0x1A5269F9, 0x1BB61DFC, 0x1BFA0C4B, 0x1B2E3E92, 0x1B622F25, 0x18E6D698, 0x18AAC72F,
        0x187EF5F6, 0x1832E441, 0x19D69044, 0x199A81F3, 0x194EB32A, 0x1902A29D, 0x1E474050,
        0x1E0B51E7, 0x1EDF633E, 0x1E937289, 0x1F77068C, 0x1F3B173B, 0x1FEF25E2, 0x1FA33455,
        0x1C27CDE8, 0x1C6BDC5F, 0x1CBFEE86, 0x1CF3FF31, 0x1D178B34, 0x1D5B9A83, 0x1D8FA85A,
        0x1DC3B9ED, 0x2608DB80, 0x2644CA37, 0x2690F8EE, 0x26DCE959, 0x27389D5C, 0x27748CEB,
        0x27A0BE32, 0x27ECAF85, 0x24685638, 0x2424478F, 0x24F07556, 0x24BC64E1, 0x255810E4,
        0x25140153, 0x25C0338A, 0x258C223D, 0x22C9C0F0, 0x2285D147, 0x2251E39E, 0x221DF229,
        0x23F9862C, 0x23B5979B, 0x2361A542, 0x232DB4F5, 0x20A94D48, 0x20E55CFF, 0x20316E26,
        0x207D7F91, 0x21990B94, 0x21D51A23, 0x210128FA, 0x214D394D, 0x2F8AED60, 0x2FC6FCD7,
        0x2F12CE0E, 0x2F5EDFB9, 0x2EBAABBC, 0x2EF6BA0B, 0x2E2288D2, 0x2E6E9965, 0x2DEA60D8,
        0x2DA6716F, 0x2D7243B6, 0x2D3E5201, 0x2CDA2604, 0x2C9637B3, 0x2C42056A, 0x2C0E14DD,
        0x2B4BF610, 0x2B07E7A7, 0x2BD3D57E, 0x2B9FC4C9, 0x2A7BB0CC, 0x2A37A17B, 0x2AE393A2,
        0x2AAF8215, 0x292B7BA8, 0x29676A1F, 0x29B358C6, 0x29FF4971, 0x281B3D74, 0x28572CC3,
        0x28831E1A, 0x28CF0FAD, 0x350CB640, 0x3540A7F7, 0x3594952E, 0x35D88499, 0x343CF09C,
        0x3470E12B, 0x34A4D3F2, 0x34E8C245, 0x376C3BF8, 0x37202A4F, 0x37F41896, 0x37B80921,
        0x365C7D24, 0x36106C93, 0x36C45E4A, 0x36884FFD, 0x31CDAD30, 0x3181BC87, 0x31558E5E,
        0x31199FE9, 0x30FDEBEC, 0x30B1FA5B, 0x3065C882, 0x3029D935, 0x33AD2088, 0x33E1313F,
        0x333503E6, 0x33791251, 0x329D6654, 0x32D177E3, 0x3205453A, 0x3249548D, 0x3C8E80A0,
        0x3CC29117, 0x3C16A3CE, 0x3C5AB279, 0x3DBEC67C, 0x3DF2D7CB, 0x3D26E512, 0x3D6AF4A5,
        0x3EEE0D18, 0x3EA21CAF, 0x3E762E76, 0x3E3A3FC1, 0x3FDE4BC4, 0x3F925A73, 0x3F4668AA,
        0x3F0A791D, 0x384F9BD0, 0x38038A67, 0x38D7B8BE, 0x389BA909, 0x397FDD0C, 0x3933CCBB,
        0x39E7FE62, 0x39ABEFD5, 0x3A2F1668, 0x3A6307DF, 0x3AB73506, 0x3AFB24B1, 0x3B1F50B4,
        0x3B534103, 0x3B8773DA, 0x3BCB626D,
    ];
}

impl Checksum {
    /// Determines [Checksum] for this message length
    pub fn from_len(mlen: usize, enhanced: bool) -> Self {
        if enhanced {
            if mlen < 128 {
                Self::XOR16
            } else if mlen < 1048575 {
                Self::XOR32
            } else {
                Self::MD5
            }
        } else {
            if mlen < 128 {
                Self::XOR8
            } else if mlen < 4096 {
                Self::XOR16
            } else if mlen < 1048575 {
                Self::XOR32
            } else {
                Self::MD5
            }
        }
    }
    /// Length we need to decode/encode this type of Checksum
    pub fn len(&self) -> usize {
        match self {
            Self::XOR8 => 1,
            Self::XOR16 => 2,
            Self::XOR32 => 4,
            Self::MD5 => 16,
        }
    }
    /// Helper to decode checksum value as unsigned 128,
    /// which covers all scenarios
    pub fn decode(&self, slice: &[u8], ck_len: usize, big_endian: bool) -> u128 {
        if ck_len == 1 {
            slice[0] as u128
        } else if ck_len == 2 {
            let val_u16 = if big_endian {
                u16::from_be_bytes([slice[0], slice[1]])
            } else {
                u16::from_le_bytes([slice[0], slice[1]])
            };
            val_u16 as u128
        } else if ck_len == 4 {
            let val_u32 = if big_endian {
                u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]])
            } else {
                u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]])
            };
            val_u32 as u128
        } else {
            unimplemented!("md5");
        }
    }
    /// Calculates expected Checksum for this msg
    pub fn calc(&self, bytes: &[u8], size: usize) -> u128 {
        match self {
            Self::XOR8 => Self::xor8_calc(bytes, size),
            Self::XOR16 => Self::xor16_calc(bytes, size),
            Self::XOR32 => Self::xor32_calc(bytes, size),
            Self::MD5 => Self::md5_calc(bytes, size),
        }
    }
    /// Calculates expected Checksum using XOR8 algorithm
    fn xor8_calc(bytes: &[u8], size: usize) -> u128 {
        let mut xor = bytes[0];
        for i in 1..size {
            xor ^= bytes[i];
        }
        xor as u128
    }
    /// Calculates expected Checksum using XOR16 algorithm
    fn xor16_calc(bytes: &[u8], size: usize) -> u128 {
        let mut crc = 0_u16;
        for i in 0..size {
            let index = (((crc >> 8) ^ bytes[i] as u16) & 0xff) as usize;
            crc = (crc << 8) ^ CRC16_TABLE[index];
            crc &= 0xffff;
        }
        crc as u128
    }
    /// Calculates expected Checksum using XO32 algorithm
    fn xor32_calc(bytes: &[u8], size: usize) -> u128 {
        let mut crc = 0_u32;
        for i in 0..size {
            let index = (((crc >> 24) ^ bytes[i] as u32) & 0xff) as usize;
            crc = (crc << 8) ^ CRC32_TABLE[index];
            crc &= 0xffffffff;
        }
        crc as u128
    }
    /// Calculates expected Checksum using MD5 algorithm
    fn md5_calc(bytes: &[u8], size: usize) -> u128 {
        let mut hasher = Md5::new();
        hasher.update(&bytes[..size]);
        let md5 = hasher.finalize();
        u128::from_le_bytes(md5.into())
    }
}

#[cfg(test)]
mod test {
    use super::Checksum;
    #[test]
    fn test_xor8() {
        let buf = [0, 1, 2, 3, 4];
        assert_eq!(Checksum::XOR8.calc(&buf, 5), 4);

        let buf = [
            0x00, 0x1f, 0x01, 0x39, 0x87, 0x20, 0x00, 0x00, 0x00, 0x17, 0x42, 0x49, 0x4e, 0x45,
            0x58, 0x20, 0x53, 0x74, 0x72, 0x65, 0x61, 0x6d, 0x20, 0x52, 0x65, 0x73, 0x74, 0x61,
            0x72, 0x74, 0x65, 0x64, 0x21,
        ];
        assert_eq!(Checksum::XOR8.calc(&buf, buf.len()), 0x84);
    }

    #[test]
    fn test_xor16() {
        // e2, 01 81
        let buf = [
            0x01, 0x81, 0x00, 0x01, 0x1d, 0x07, 0xf6, 0x00, 0x03, 0xd8, 0x72, 0x00, 0x03, 0xf4,
            0x80, 0x31, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0xac,
            0xdc, 0x00, 0x00, 0xb8, 0x38, 0xc3, 0x00, 0x00, 0x00, 0x00, 0x20, 0x30, 0xd5, 0x5c,
            0x00, 0xbf, 0xf8, 0x96, 0x4c, 0x65, 0x6e, 0xda, 0x41, 0x3f, 0x6d, 0x97, 0xd5, 0xd0,
            0x00, 0x00, 0x00, 0x40, 0xb4, 0x21, 0xa2, 0x39, 0x40, 0x00, 0x00, 0x32, 0x20, 0x00,
            0x00, 0x43, 0x44, 0xf8, 0x00, 0xb3, 0x18, 0x00, 0x00, 0x42, 0x78, 0x60, 0x00, 0x36,
            0x49, 0xa0, 0x00, 0x37, 0x16, 0x60, 0x00, 0x40, 0x02, 0xa8, 0x2c, 0x0b, 0x2a, 0x18,
            0x0c, 0xc0, 0x08, 0x23, 0xb8, 0x97, 0xbd, 0xf9, 0x99, 0x3f, 0xee, 0x23, 0x55, 0xce,
            0x2e, 0x11, 0x70, 0xb1, 0x31, 0xa4, 0x00, 0xad, 0xac, 0x00, 0x00, 0x41, 0xa0, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x04,
        ];

        assert_eq!(Checksum::XOR16.calc(&buf, buf.len()), 0x7d49);

        let buf = [
            0x01, 0x81, 0x00, 0x01, 0x07, 0x07, 0xf6, 0x00, 0x03, 0xd8, 0x72, 0x00, 0x03, 0xf4,
            0x80, 0x31, 0xb0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1d, 0x00, 0x00, 0x00, 0x00, 0xab,
            0xc0, 0x00, 0x00, 0xb9, 0x09, 0x3b, 0x60, 0x00, 0x00, 0x00, 0x1d, 0x30, 0xc3, 0x30,
            0x00, 0x3f, 0xf9, 0xa2, 0xc9, 0x26, 0x53, 0xc2, 0x7b, 0x3f, 0x71, 0x2c, 0xe0, 0xd8,
            0x00, 0x00, 0x00, 0x40, 0xb4, 0x21, 0xb0, 0xf1, 0x60, 0x00, 0x00, 0x33, 0xa0, 0x00,
            0x00, 0x43, 0x98, 0x64, 0x00, 0xb2, 0x60, 0x00, 0x00, 0xc2, 0x2f, 0xa0, 0x00, 0xb6,
            0x0c, 0xe0, 0x00, 0x36, 0x83, 0xc0, 0x00, 0xbf, 0xfe, 0xa8, 0x9a, 0xfb, 0x49, 0x69,
            0xb2, 0xbf, 0xd7, 0x73, 0x3f, 0x12, 0x4a, 0xa8, 0x69, 0x3f, 0xef, 0x09, 0xab, 0xae,
            0x21, 0x65, 0xd4, 0xb1, 0x33, 0xe8, 0x00, 0xae, 0xa1, 0xc0, 0x00, 0x41, 0xa0, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x04,
        ];

        assert_eq!(Checksum::XOR16.calc(&buf, buf.len()), 0x6c23);

        let buf = [
            0x01, 0x81, 0x00, 0x01, 0x06, 0x07, 0xf6, 0x00, 0x03, 0xd8, 0x72, 0x00, 0x03, 0xf4,
            0x80, 0xb2, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x53, 0x00, 0x00, 0x00, 0x00, 0xac,
            0xfc, 0x00, 0x00, 0x38, 0x31, 0xab, 0x80, 0x00, 0x00, 0x00, 0x53, 0x30, 0xc5, 0x24,
            0x00, 0xbf, 0xf8, 0xbb, 0x57, 0x3d, 0x09, 0x5f, 0x9d, 0x3f, 0x88, 0xee, 0x38, 0x68,
            0x00, 0x00, 0x00, 0x40, 0xb4, 0x21, 0x9d, 0xac, 0xc0, 0x00, 0x00, 0x34, 0x5a, 0x00,
            0x00, 0x43, 0x48, 0x50, 0x00, 0xb3, 0xe4, 0x00, 0x00, 0x42, 0x67, 0x40, 0x00, 0x36,
            0x42, 0x80, 0x00, 0x37, 0x16, 0xe8, 0x00, 0x40, 0x02, 0x6a, 0xdc, 0xf8, 0x6b, 0x10,
            0x56, 0xc0, 0x03, 0xd3, 0x59, 0xfa, 0x22, 0x6d, 0x54, 0x3f, 0xee, 0x99, 0xf5, 0x34,
            0x32, 0xf3, 0xdd, 0xb1, 0x2b, 0xf6, 0x00, 0xac, 0xe8, 0x00, 0x00, 0x41, 0xa0, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x04,
        ];

        assert_eq!(Checksum::XOR16.calc(&buf, buf.len()), 0x1919);

        let buf = [
            0x01, 0x81, 0x00, 0x01, 0x11, 0x07, 0xf6, 0x00, 0x03, 0xe5, 0x74, 0x00, 0x03, 0xf4,
            0x80, 0xb1, 0xd0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x68, 0x00, 0x00, 0x00, 0x00, 0x2c,
            0x84, 0x00, 0x00, 0x37, 0xae, 0x23, 0x00, 0x00, 0x00, 0x00, 0x68, 0x30, 0xc9, 0xa4,
            0x00, 0xbf, 0xf2, 0x1b, 0xb8, 0xd2, 0xf3, 0x07, 0xe2, 0x3f, 0x8e, 0xbe, 0xca, 0x08,
            0x00, 0x00, 0x00, 0x40, 0xb4, 0x21, 0xbe, 0x8a, 0x80, 0x00, 0x00, 0xb3, 0xc0, 0x00,
            0x00, 0x43, 0x53, 0x30, 0x00, 0x34, 0xb4, 0x00, 0x00, 0xc2, 0x64, 0xa0, 0x00, 0xb6,
            0x3f, 0xa0, 0x00, 0x37, 0x0e, 0xf8, 0x00, 0xbf, 0xec, 0xde, 0x8f, 0x8b, 0x25, 0x86,
            0x86, 0x3f, 0xf5, 0xf8, 0xf4, 0xf0, 0x6d, 0x38, 0x8e, 0x3f, 0xee, 0x7c, 0xb1, 0xdf,
            0xad, 0x10, 0xdf, 0xb1, 0x34, 0x46, 0x00, 0xaa, 0x80, 0x00, 0x00, 0x41, 0xa0, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x04,
        ];

        assert_eq!(Checksum::XOR16.calc(&buf, buf.len()), 0x72d8);

        let buf = [
            0x01, 0x81, 0x00, 0x01, 0x00, 0x07, 0xf6, 0x00, 0x03, 0xf2, 0x6a, 0x00, 0x03, 0xf4,
            0x80, 0x31, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5b, 0x00, 0x00, 0x00, 0x00, 0xac,
            0xec, 0x00, 0x00, 0xb9, 0x21, 0x70, 0x60, 0x00, 0x00, 0x00, 0x5b, 0x30, 0xb1, 0xd4,
            0x00, 0xbf, 0xed, 0x6f, 0x04, 0x75, 0x35, 0xbf, 0x7c, 0x3f, 0x81, 0x09, 0xb5, 0x7c,
            0x00, 0x00, 0x00, 0x40, 0xb4, 0x21, 0xa9, 0xec, 0xa0, 0x00, 0x00, 0x34, 0x3c, 0x00,
            0x00, 0x43, 0x55, 0xd8, 0x00, 0x33, 0xec, 0x00, 0x00, 0xc2, 0x6f, 0xc0, 0x00, 0xb6,
            0x55, 0x80, 0x00, 0x37, 0x16, 0x80, 0x00, 0xbf, 0xeb, 0x2f, 0xea, 0x1a, 0x2d, 0x94,
            0x4f, 0x3f, 0xe5, 0xf2, 0x7f, 0x44, 0xff, 0xc4, 0xc1, 0x3f, 0xef, 0x2d, 0x2d, 0x85,
            0x03, 0xc4, 0xfb, 0xb1, 0x2c, 0x40, 0x00, 0x2b, 0x00, 0x00, 0x00, 0x41, 0xa0, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x04,
        ];

        assert_eq!(Checksum::XOR16.calc(&buf, buf.len()), 0x5376);
    }
}