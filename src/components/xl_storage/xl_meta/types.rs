/// 基础类型定义 - 对应 MinIO 的 enum 定义

use serde::{Deserialize, Serialize};

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
