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
    pub etag: String,
    #[serde(default)]
    pub content_type: String,
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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum StoragePathKind {
    #[default]
    Final,
    Temporary,
    Config,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum StorageWriteKind {
    #[default]
    Data,
    Metadata,
    Config,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateFileOptions {
    pub path_kind: StoragePathKind,
    pub write_kind: StorageWriteKind,
    pub fsync: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WriteAllOptions {
    pub path_kind: StoragePathKind,
    pub write_kind: StorageWriteKind,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenameDataOptions {
    pub path_kind: StoragePathKind,
    pub defer_old_data_dir_cleanup: bool,
    pub defer_src_path_cleanup: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenameDataResult {
    pub old_data_dir: Option<String>,
    pub old_data_path: Option<String>,
    pub cleanup_src_volume: String,
    pub cleanup_src_path: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeletePathOptions {
    pub recursive: bool,
    pub ignore_not_found: bool,
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
