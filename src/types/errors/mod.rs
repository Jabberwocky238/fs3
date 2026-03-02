mod engine;
mod metadata_storage;
mod mount;

pub use engine::S3EngineError;
pub use metadata_storage::S3MetadataStorageError;
pub use mount::S3MountError;

impl From<S3MetadataStorageError> for S3EngineError {
    fn from(e: S3MetadataStorageError) -> Self {
        Self::Storage(e.to_string())
    }
}

impl From<S3MountError> for S3EngineError {
    fn from(e: S3MountError) -> Self {
        Self::Mount(e.to_string())
    }
}
