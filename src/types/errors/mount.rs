use crate::types::traits::s3_handler::S3HandlerBridgeError;

#[derive(Debug, thiserror::Error)]
pub enum S3MountError {
    #[error("no such bucket: {0}")]
    NoSuchBucket(String),
    #[error("no such key: {bucket}/{key}")]
    NoSuchKey { bucket: String, key: String },
    #[error("invalid key: {0}")]
    BadKey(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("{0}")]
    HandlerBridge(#[from] S3HandlerBridgeError),
}

impl From<std::io::Error> for S3MountError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}
