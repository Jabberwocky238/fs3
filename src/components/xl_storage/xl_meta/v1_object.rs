/// Legacy V1 object - 对应 MinIO xlMetaV1Object

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV1Object {
    #[serde(rename = "version")]
    pub version: String,
    #[serde(rename = "format")]
    pub format: String,
    #[serde(rename = "stat")]
    pub stat: StatInfo,
    #[serde(rename = "erasure")]
    pub erasure: ErasureInfo,
    #[serde(rename = "minio")]
    pub minio: MinioInfo,
    #[serde(rename = "meta", skip_serializing_if = "HashMap::is_empty", default)]
    pub meta: HashMap<String, String>,
    #[serde(rename = "parts", skip_serializing_if = "Vec::is_empty", default)]
    pub parts: Vec<ObjectPartInfo>,
    #[serde(rename = "versionId", skip_serializing_if = "Option::is_none", default)]
    pub version_id: Option<String>,
    #[serde(rename = "dataDir", skip_serializing_if = "Option::is_none", default)]
    pub data_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatInfo {
    #[serde(rename = "size")]
    pub size: i64,
    #[serde(rename = "modTime")]
    pub mod_time: i64,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "dir")]
    pub dir: bool,
    #[serde(rename = "mode")]
    pub mode: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureInfo {
    #[serde(rename = "algorithm")]
    pub algorithm: String,
    #[serde(rename = "data")]
    pub data_blocks: i32,
    #[serde(rename = "parity")]
    pub parity_blocks: i32,
    #[serde(rename = "blockSize")]
    pub block_size: i64,
    #[serde(rename = "index")]
    pub index: i32,
    #[serde(rename = "distribution")]
    pub distribution: Vec<i32>,
    #[serde(rename = "checksum", skip_serializing_if = "Vec::is_empty", default)]
    pub checksums: Vec<ChecksumInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumInfo {
    #[serde(rename = "PartNumber")]
    pub part_number: i32,
    #[serde(rename = "Algorithm")]
    pub algorithm: BitrotAlgorithm,
    #[serde(rename = "Hash")]
    pub hash: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u32)]
pub enum BitrotAlgorithm {
    SHA256 = 1,
    HighwayHash256 = 2,
    HighwayHash256S = 3,
    BLAKE2b512 = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPartInfo {
    #[serde(rename = "etag", skip_serializing_if = "Option::is_none", default)]
    pub etag: Option<String>,
    #[serde(rename = "number")]
    pub number: i32,
    #[serde(rename = "size")]
    pub size: i64,
    #[serde(rename = "actualSize")]
    pub actual_size: i64,
    #[serde(rename = "modTime")]
    pub mod_time: i64,
    #[serde(rename = "index", skip_serializing_if = "Option::is_none", default)]
    pub index: Option<Vec<u8>>,
    #[serde(rename = "crc", skip_serializing_if = "HashMap::is_empty", default)]
    pub checksums: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinioInfo {
    #[serde(rename = "release")]
    pub release: String,
}
