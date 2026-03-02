use async_trait::async_trait;

use crate::types::s3::core::*;

use super::s3_engine::S3EngineError;

/// Trait for mounting object data to a file system.
/// Handles reading/writing the actual bytes of objects.
/// Metadata is handled separately by S3MetadataStorage.
#[async_trait]
pub trait S3MountRead<E: S3EngineError> {
    async fn read_object(&self, bucket: &str, key: &str) -> Result<BoxByteStream, E>;
    async fn read_object_range(&self, bucket: &str, key: &str, range: &str) -> Result<BoxByteStream, E>;
    async fn object_exists(&self, bucket: &str, key: &str) -> Result<bool, E>;
    async fn object_size(&self, bucket: &str, key: &str) -> Result<u64, E>;
}

#[async_trait]
pub trait S3MountWrite<E: S3EngineError> {
    async fn write_object(&self, bucket: &str, key: &str, body: BoxByteStream) -> Result<u64, E>;
    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), E>;
    async fn copy_object(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str) -> Result<u64, E>;
}

#[async_trait]
pub trait S3MountBucket<E: S3EngineError> {
    async fn create_bucket_dir(&self, bucket: &str) -> Result<(), E>;
    async fn delete_bucket_dir(&self, bucket: &str) -> Result<(), E>;
    async fn bucket_dir_exists(&self, bucket: &str) -> Result<bool, E>;
}

#[async_trait]
pub trait S3MountMultipart<E: S3EngineError> {
    async fn write_part(&self, bucket: &str, key: &str, upload_id: &str, part_number: u32, body: BoxByteStream) -> Result<u64, E>;
    async fn assemble_parts(&self, bucket: &str, key: &str, upload_id: &str, parts: &[UploadedPart]) -> Result<u64, E>;
    async fn cleanup_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<(), E>;
}

pub trait S3Mount<E: S3EngineError>:
    S3MountRead<E> + S3MountWrite<E> + S3MountBucket<E> + S3MountMultipart<E>
{
}

impl<T, E> S3Mount<E> for T
where
    E: S3EngineError,
    T: S3MountRead<E> + S3MountWrite<E> + S3MountBucket<E> + S3MountMultipart<E>,
{
}
