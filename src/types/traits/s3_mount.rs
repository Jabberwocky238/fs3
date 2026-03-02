use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::errors::S3MountError;

/// Trait for mounting object data to a file system.
/// Handles reading/writing the actual bytes of objects.
/// Metadata is handled separately by S3MetadataStorage.
#[async_trait]
pub trait S3MountRead {
    async fn read_object(&self, bucket: &str, key: &str) -> Result<BoxByteStream, S3MountError>;
    async fn read_object_range(&self, bucket: &str, key: &str, range: &str) -> Result<BoxByteStream, S3MountError>;
    async fn object_exists(&self, bucket: &str, key: &str) -> Result<bool, S3MountError>;
    async fn object_size(&self, bucket: &str, key: &str) -> Result<u64, S3MountError>;
}

#[async_trait]
pub trait S3MountWrite {
    async fn write_object(&self, bucket: &str, key: &str, body: BoxByteStream) -> Result<u64, S3MountError>;
    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), S3MountError>;
    async fn copy_object(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str) -> Result<u64, S3MountError>;
}

#[async_trait]
pub trait S3MountBucket {
    async fn create_bucket_dir(&self, bucket: &str) -> Result<(), S3MountError>;
    async fn delete_bucket_dir(&self, bucket: &str) -> Result<(), S3MountError>;
    async fn bucket_dir_exists(&self, bucket: &str) -> Result<bool, S3MountError>;
}

#[async_trait]
pub trait S3MountMultipart {
    async fn write_part(&self, bucket: &str, key: &str, upload_id: &str, part_number: u32, body: BoxByteStream) -> Result<u64, S3MountError>;
    async fn assemble_parts(&self, bucket: &str, key: &str, upload_id: &str, parts: &[UploadedPart]) -> Result<u64, S3MountError>;
    async fn cleanup_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<(), S3MountError>;
}

pub trait S3Mount:
    S3MountRead + S3MountWrite + S3MountBucket + S3MountMultipart
{
}

impl<T> S3Mount for T
where
    T: S3MountRead + S3MountWrite + S3MountBucket + S3MountMultipart,
{
}
