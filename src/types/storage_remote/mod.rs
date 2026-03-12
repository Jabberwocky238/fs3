use serde::{Deserialize, Serialize};

use crate::types::s3::storage_types::{
    CreateFileOptions, DeletePathOptions, FileInfo, RenameDataOptions, RenameDataResult,
    WriteAllOptions,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRemoteError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFileRequest {
    pub volume: String,
    pub path: String,
    pub size: i64,
    pub opts: CreateFileOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFileResponse {
    pub written: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadVersionRequest {
    pub volume: String,
    pub path: String,
    pub version_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadVersionResponse {
    pub file_info: FileInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteAllRequest {
    pub volume: String,
    pub path: String,
    pub data: Vec<u8>,
    pub opts: WriteAllOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteAllResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameDataRequest {
    pub src_volume: String,
    pub src_path: String,
    pub file_info: FileInfo,
    pub dst_volume: String,
    pub dst_path: String,
    pub opts: RenameDataOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameDataResponse {
    pub result: RenameDataResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePathRequest {
    pub volume: String,
    pub path: String,
    pub opts: DeletePathOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePathResponse;
