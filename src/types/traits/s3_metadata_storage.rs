use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::s3::metadata_storage_error::S3MetadataStorageError;

type Result<T> = std::result::Result<T, S3MetadataStorageError>;

/// Trait for persisting S3 metadata (buckets, objects, multipart uploads).
/// Implementations handle storage to databases, files, etc.
/// This does NOT handle object data — only metadata.
#[async_trait]
pub trait S3MetadataStorageBucket {
    async fn store_bucket(&self, bucket: &S3Bucket) -> Result<()>;
    async fn load_bucket(&self, name: &str) -> Result<Option<S3Bucket>>;
    async fn list_buckets(&self) -> Result<Vec<S3Bucket>>;
    async fn delete_bucket(&self, name: &str) -> Result<()>;
    async fn store_bucket_metadata(&self, bucket: &str, metadata: &BucketMetadataBundle) -> Result<()>;
    async fn load_bucket_metadata(&self, bucket: &str) -> Result<Option<BucketMetadataBundle>>;
}

#[async_trait]
pub trait S3MetadataStorageObject {
    async fn store_object_meta(&self, obj: &S3Object) -> Result<()>;
    async fn load_object_meta(&self, bucket: &str, key: &str) -> Result<Option<S3Object>>;
    async fn delete_object_meta(&self, bucket: &str, key: &str) -> Result<()>;
    async fn list_objects(&self, bucket: &str, options: &ListOptions) -> Result<ObjectListPage>;
}

#[async_trait]
pub trait S3MetadataStorageMultipart {
    async fn store_multipart(&self, upload: &MultipartUpload) -> Result<()>;
    async fn load_multipart(&self, upload_id: &str) -> Result<Option<MultipartUpload>>;
    async fn delete_multipart(&self, upload_id: &str) -> Result<()>;
    async fn store_uploaded_part(&self, upload_id: &str, part: &UploadedPart) -> Result<()>;
    async fn list_uploaded_parts(&self, upload_id: &str) -> Result<Vec<UploadedPart>>;
    async fn list_multipart_uploads(&self, bucket: &str) -> Result<Vec<MultipartUpload>>;
}

pub trait S3MetadataStorage:
    S3MetadataStorageBucket + S3MetadataStorageObject + S3MetadataStorageMultipart
{
}

impl<T> S3MetadataStorage for T
where
    T: S3MetadataStorageBucket + S3MetadataStorageObject + S3MetadataStorageMultipart,
{
}
