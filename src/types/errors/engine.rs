use crate::types::traits::s3_handler::S3HandlerBridgeError;

#[derive(Debug, thiserror::Error)]
pub enum S3EngineError {
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
    #[error("invalid policy: {0}")]
    InvalidPolicy(String),
    #[error("NoSuchCORSConfiguration")]
    NoSuchCORSConfiguration,
    #[error("internal error: {0}")]
    Internal(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("mount error: {0}")]
    Mount(String),
    #[error("{0}")]
    HandlerBridge(#[from] S3HandlerBridgeError),
}

