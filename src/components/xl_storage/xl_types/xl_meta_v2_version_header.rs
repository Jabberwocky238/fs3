use rmp::encode::{write_array_len, write_bin, write_sint, write_uint};
use rmpv::decode::read_value;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use super::xl_meta_v2_version::VersionType;

#[derive(Debug, Clone, Copy)]
pub struct XlFlags(u8);

impl XlFlags {
    pub const FREE_VERSION: u8 = 1 << 0;
    pub const USES_DATA_DIR: u8 = 1 << 1;
    pub const INLINE_DATA: u8 = 1 << 2;

    pub fn new(value: u8) -> Self {
        Self(value)
    }

    pub fn has_free_version(&self) -> bool {
        self.0 & Self::FREE_VERSION != 0
    }

    pub fn has_uses_data_dir(&self) -> bool {
        self.0 & Self::USES_DATA_DIR != 0
    }

    pub fn has_inline_data(&self) -> bool {
        self.0 & Self::INLINE_DATA != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV2VersionHeader {
    #[serde(rename = "vid")]
    pub version_id: [u8; 16],
    #[serde(rename = "mt")]
    pub mod_time: i64,
    #[serde(rename = "sig")]
    pub signature: [u8; 4],
    #[serde(rename = "vt")]
    pub version_type: u8,
    #[serde(rename = "f")]
    pub flags: u8,
    #[serde(rename = "n")]
    pub ec_n: u8,
    #[serde(rename = "m")]
    pub ec_m: u8,
}

impl From<Vec<u8>> for XlMetaV2VersionHeader {
    fn from(bytes: Vec<u8>) -> Self {
        let value = read_value(&mut Cursor::new(&bytes)).unwrap();
        if let Some(items) = value.as_array() {
            return Self {
                version_id: items[0].as_slice().unwrap().try_into().unwrap(),
                mod_time: items[1].as_i64().unwrap(),
                signature: items[2].as_slice().unwrap().try_into().unwrap(),
                version_type: items[3].as_u64().unwrap() as u8,
                flags: items[4].as_u64().unwrap() as u8,
                ec_n: items[5].as_u64().unwrap() as u8,
                ec_m: items[6].as_u64().unwrap() as u8,
            };
        }
        rmp_serde::from_slice(&bytes).unwrap()
    }
}

impl From<XlMetaV2VersionHeader> for Vec<u8> {
    fn from(val: XlMetaV2VersionHeader) -> Self {
        let mut buf = Vec::new();
        write_array_len(&mut buf, 7).unwrap();
        write_bin(&mut buf, &val.version_id).unwrap();
        write_sint(&mut buf, val.mod_time).unwrap();
        write_bin(&mut buf, &val.signature).unwrap();
        write_uint(&mut buf, val.version_type as u64).unwrap();
        write_uint(&mut buf, val.flags as u64).unwrap();
        write_uint(&mut buf, val.ec_n as u64).unwrap();
        write_uint(&mut buf, val.ec_m as u64).unwrap();
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_1() {
        let expected = hex::decode(
            "97c4100102030405060708090a0b0c0d0e0f10d3499602d200000000c404786c322001000402",
        )
        .unwrap();
        let decoded: XlMetaV2VersionHeader = expected.clone().into();
        assert_eq!(
            decoded.version_id,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
        assert_eq!(decoded.mod_time, 1234567890);
        assert_eq!(decoded.version_type, 1);
        assert_eq!(decoded.ec_n, 4);
        assert_eq!(decoded.ec_m, 2);

        let encoded: Vec<u8> = decoded.into();
        assert_eq!(encoded, expected);
    }
}
