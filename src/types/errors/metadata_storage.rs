use crate::types::traits::s3_handler::S3HandlerBridgeError;

#[derive(Debug, thiserror::Error)]
pub enum S3MetadataStorageError {
    #[error("bucket not found: {0}")]
    BucketNotFound(String),
    #[error("object not found: {bucket}/{key}")]
    ObjectNotFound { bucket: String, key: String },
    #[error("multipart not found: {0}")]
    MultipartNotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    HandlerBridge(#[from] S3HandlerBridgeError),
    #[cfg(feature = "storage-sqlite")]
    #[error("sqlite error: {0}")]
    Sqlite(#[from] sqlx::Error),
    #[error("{0}")]
    Other(String),
}
