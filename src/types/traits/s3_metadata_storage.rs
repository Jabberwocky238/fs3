use async_trait::async_trait;

use crate::types::s3::core::*;

use super::s3_engine::S3EngineError;

/// Trait for persisting S3 metadata (buckets, objects, multipart uploads).
/// Implementations handle storage to databases, files, etc.
/// This does NOT handle object data — only metadata.
#[async_trait]
pub trait S3MetadataStorageBucket<E: S3EngineError> {
    async fn store_bucket(&self, bucket: &S3Bucket) -> Result<(), E>;
    async fn load_bucket(&self, name: &str) -> Result<Option<S3Bucket>, E>;
    async fn list_buckets(&self) -> Result<Vec<S3Bucket>, E>;
    async fn delete_bucket(&self, name: &str) -> Result<(), E>;
    async fn store_bucket_metadata(&self, bucket: &str, metadata: &BucketMetadataBundle) -> Result<(), E>;
    async fn load_bucket_metadata(&self, bucket: &str) -> Result<Option<BucketMetadataBundle>, E>;
}

#[async_trait]
pub trait S3MetadataStorageObject<E: S3EngineError> {
    async fn store_object_meta(&self, obj: &S3Object) -> Result<(), E>;
    async fn load_object_meta(&self, bucket: &str, key: &str) -> Result<Option<S3Object>, E>;
    async fn delete_object_meta(&self, bucket: &str, key: &str) -> Result<(), E>;
    async fn list_objects(&self, bucket: &str, options: &ListOptions) -> Result<ObjectListPage, E>;
}

#[async_trait]
pub trait S3MetadataStorageMultipart<E: S3EngineError> {
    async fn store_multipart(&self, upload: &MultipartUpload) -> Result<(), E>;
    async fn load_multipart(&self, upload_id: &str) -> Result<Option<MultipartUpload>, E>;
    async fn delete_multipart(&self, upload_id: &str) -> Result<(), E>;
    async fn store_uploaded_part(&self, upload_id: &str, part: &UploadedPart) -> Result<(), E>;
    async fn list_uploaded_parts(&self, upload_id: &str) -> Result<Vec<UploadedPart>, E>;
    async fn list_multipart_uploads(&self, bucket: &str) -> Result<Vec<MultipartUpload>, E>;
}

pub trait S3MetadataStorage<E: S3EngineError>:
    S3MetadataStorageBucket<E> + S3MetadataStorageObject<E> + S3MetadataStorageMultipart<E>
{
}

impl<T, E> S3MetadataStorage<E> for T
where
    E: S3EngineError,
    T: S3MetadataStorageBucket<E> + S3MetadataStorageObject<E> + S3MetadataStorageMultipart<E>,
{
}
