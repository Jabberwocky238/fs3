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
    pub version_id: [u8; 16],
    pub mod_time: i64,
    pub signature: [u8; 4],
    pub version_type: VersionType,
    pub flags: u8,
    pub ec_n: u8,
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
