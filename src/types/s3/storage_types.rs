use serde::{Deserialize, Serialize};
use crate::types::s3::core::BoxByteStream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolInfo {
    pub name: String,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub volume: String,
    pub name: String,
    pub version_id: String,
    pub size: u64,
    pub data_dir: String,
    #[serde(default)]
    pub user_metadata: std::collections::HashMap<String, String>,
    #[serde(default = "default_one")]
    pub erasure_index: i32,
    #[serde(default = "default_one")]
    pub erasure_m: i32,
    #[serde(default)]
    pub erasure_n: i32,
}

fn default_one() -> i32 { 1 }

pub struct PutObjReader {
    pub reader: BoxByteStream,
    pub size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartInfo {
    pub part_number: u32,
    pub etag: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletePart {
    pub part_number: u32,
    pub etag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMultipartUploadResult {
    pub upload_id: String,
}
