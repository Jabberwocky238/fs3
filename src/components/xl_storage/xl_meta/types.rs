/// 基础类型定义 - 对应 MinIO 的 enum 定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const XL_HEADER: [u8; 4] = *b"XL2 ";
pub const XL_VERSION_MAJOR: u16 = 1;
pub const XL_VERSION_MINOR: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VersionType {
    Invalid = 0,
    Object = 1,
    Delete = 2,
    Legacy = 3,
}

impl VersionType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Object,
            2 => Self::Delete,
            3 => Self::Legacy,
            _ => Self::Invalid,
        }
    }
}

impl serde::Serialize for VersionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> serde::Deserialize<'de> for VersionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = u8::deserialize(deserializer)?;
        Ok(match v {
            0 => VersionType::Invalid,
            1 => VersionType::Object,
            2 => VersionType::Delete,
            3 => VersionType::Legacy,
            _ => VersionType::Invalid,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ErasureAlgo {
    Invalid = 0,
    ReedSolomon = 1,
}

impl serde::Serialize for ErasureAlgo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> serde::Deserialize<'de> for ErasureAlgo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = u8::deserialize(deserializer)?;
        Ok(match v {
            0 => ErasureAlgo::Invalid,
            1 => ErasureAlgo::ReedSolomon,
            _ => ErasureAlgo::Invalid,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChecksumAlgo {
    Invalid = 0,
    HighwayHash = 1,
}

impl serde::Serialize for ChecksumAlgo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> serde::Deserialize<'de> for ChecksumAlgo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = u8::deserialize(deserializer)?;
        Ok(match v {
            0 => ChecksumAlgo::Invalid,
            1 => ChecksumAlgo::HighwayHash,
            _ => ChecksumAlgo::Invalid,
        })
    }
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
