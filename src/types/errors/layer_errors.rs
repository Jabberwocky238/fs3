use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Volume not found: {0}")]
    VolumeNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("IO error: {0}")]
    Io(String),
}

#[derive(Debug, Error)]
pub enum S3Error {
    #[error("Bucket not found: {0}")]
    NoSuchBucket(String),

    #[error("Object not found: {0}")]
    NoSuchKey(String),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}
