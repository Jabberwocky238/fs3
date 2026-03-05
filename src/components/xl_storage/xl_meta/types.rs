/// 基础类型定义 - 对应 MinIO 的 enum 定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const XL_HEADER: [u8; 4] = *b"XL2 ";
pub const XL_VERSION_MAJOR: u16 = 1;
pub const XL_VERSION_MINOR: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum VersionType {
    Invalid = 0,
    Object = 1,
    Delete = 2,
    Legacy = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ErasureAlgo {
    Invalid = 0,
    ReedSolomon = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ChecksumAlgo {
    Invalid = 0,
    HighwayHash = 1,
}

pub type XlMetaInlineData = Vec<u8>;

pub fn hash_deterministic_string(m: &HashMap<String, String>) -> u64 {
    let mut crc = 0xc2b40bbac11a7295u64;
    for (k, v) in m {
        crc ^= (xxhash_rust::xxh3::xxh3_64(k.as_bytes()) ^ 0x4ee3bbaf7ab2506b)
             + (xxhash_rust::xxh3::xxh3_64(v.as_bytes()) ^ 0x8da4c8da66194257);
    }
    crc
}
