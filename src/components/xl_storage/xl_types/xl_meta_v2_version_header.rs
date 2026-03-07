use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

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
        rmp_serde::from_slice(&bytes).unwrap()
    }
}

impl From<XlMetaV2VersionHeader> for Vec<u8> {
    fn from(val: XlMetaV2VersionHeader) -> Self {
        rmp_serde::to_vec(&val).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_1() {
        let expected = hex::decode("87a3766964c4100102030405060708090a0b0c0d0e0f10a26d74d2499602d2a3736967c404786c3220a2767401a16600a16e04a16d02").unwrap();
        let decoded: XlMetaV2VersionHeader = expected.clone().into();
        assert_eq!(decoded.version_id, [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
        assert_eq!(decoded.mod_time, 1234567890);
        assert_eq!(decoded.version_type, 1);
        assert_eq!(decoded.ec_n, 4);
        assert_eq!(decoded.ec_m, 2);

        let encoded: Vec<u8> = decoded.into();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_case_2() {
        let expected = hex::decode("87a3766964c410ffeeddccbbaa99887766554433221100a26d74d2499602d3a3736967c404786c3220a2767402a16603a16e02a16d01").unwrap();
        let decoded: XlMetaV2VersionHeader = expected.clone().into();
        assert_eq!(decoded.version_type, 2);
        assert_eq!(decoded.flags, 3);
        assert_eq!(decoded.ec_n, 2);
        assert_eq!(decoded.ec_m, 1);
    }
}
