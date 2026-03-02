use chrono::Utc;
use uuid::Uuid;

use crate::types::s3::core::*;
use crate::types::traits::s3_handler::S3HandlerBridgeError;

mod bucket;
mod config;
mod multipart;
mod object;

#[derive(Debug, thiserror::Error)]
pub enum S3EngineImplError {
    #[error("bucket not found: {0}")]
    BucketNotFound(String),
    #[error("bucket already exists: {0}")]
    BucketAlreadyExists(String),
    #[error("bucket is not empty: {0}")]
    BucketNotEmpty(String),
    #[error("object not found: {bucket}/{key}")]
    ObjectNotFound { bucket: String, key: String },
    #[error("object version not found: {bucket}/{key}:{version_id}")]
    ObjectVersionNotFound { bucket: String, key: String, version_id: String },
    #[error("multipart upload not found: {0}")]
    MultipartNotFound(String),
    #[error("multipart part missing: upload_id={upload_id} part_number={part_number}")]
    MultipartPartMissing { upload_id: String, part_number: u32 },
    #[error("invalid range header: {0}")]
    InvalidRange(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("mount error: {0}")]
    Mount(String),
    #[error("{0}")]
    HandlerBridge(#[from] S3HandlerBridgeError),
}

impl From<crate::components::s3_metadata_storage::memory::MemoryMetadataStorageError> for S3EngineImplError {
    fn from(e: crate::components::s3_metadata_storage::memory::MemoryMetadataStorageError) -> Self {
        Self::Storage(e.to_string())
    }
}

impl From<crate::components::s3_mount::memory::MemoryMountError> for S3EngineImplError {
    fn from(e: crate::components::s3_mount::memory::MemoryMountError) -> Self {
        Self::Mount(e.to_string())
    }
}

pub struct S3EngineImpl<S, M> {
    pub metadata: S,
    pub mount: M,
}

impl<S, M> S3EngineImpl<S, M> {
    pub fn new(metadata: S, mount: M) -> Self {
        Self { metadata, mount }
    }

    fn new_version_ref() -> ObjectVersionRef {
        ObjectVersionRef {
            version_id: Some(Uuid::new_v4().to_string()),
            is_latest: true,
            delete_marker: false,
        }
    }

    fn compute_etag(data: &[u8]) -> String {
        format!("{:x}", md5::compute(data))
    }

    fn now_doc(body: String) -> TimedDocument {
        TimedDocument { body, updated_at: Utc::now() }
    }
}
