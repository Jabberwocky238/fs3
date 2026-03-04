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
}

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
