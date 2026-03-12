use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::traits::StdError;

#[async_trait]
pub trait S3ObjectTaggingEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_object_tagging(&self, bucket: &str, key: &str) -> Result<TagMap, E>;
    async fn put_object_tagging(&self, bucket: &str, key: &str, tags: TagMap) -> Result<(), E>;
    async fn delete_object_tagging(&self, bucket: &str, key: &str) -> Result<(), E>;
}

#[async_trait]
pub trait S3ObjectRetentionEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_object_retention(&self, bucket: &str, key: &str) -> Result<Option<ObjectRetention>, E>;
    async fn put_object_retention(&self, bucket: &str, key: &str, retention: ObjectRetention) -> Result<(), E>;
}

#[async_trait]
pub trait S3ObjectLegalHoldEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_object_legal_hold(&self, bucket: &str, key: &str) -> Result<Option<ObjectLegalHold>, E>;
    async fn put_object_legal_hold(&self, bucket: &str, key: &str, legal_hold: ObjectLegalHold) -> Result<(), E>;
}

#[async_trait]
pub trait S3BucketLifecycleEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_bucket_lifecycle(&self, bucket: &str) -> Result<Vec<String>, E>;
    async fn put_bucket_lifecycle(&self, bucket: &str, rules: Vec<String>) -> Result<(), E>;
    async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<(), E>;
}

#[async_trait]
pub trait S3BucketEncryptionEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_bucket_encryption(&self, bucket: &str) -> Result<Option<BucketEncryption>, E>;
    async fn put_bucket_encryption(&self, bucket: &str, algorithm: String, key_id: Option<String>) -> Result<(), E>;
    async fn delete_bucket_encryption(&self, bucket: &str) -> Result<(), E>;
}

#[async_trait]
pub trait S3BucketObjectLockEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_bucket_object_lock_config(&self, bucket: &str) -> Result<Option<BucketObjectLockConfig>, E>;
    async fn put_bucket_object_lock_config(&self, bucket: &str, enabled: bool, mode: Option<String>, days: Option<u32>, years: Option<u32>) -> Result<(), E>;
}

#[async_trait]
pub trait S3BucketVersionEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<BucketVersioning>, E>;
    async fn put_bucket_versioning(&self, bucket: &str, status: String, mfa_delete: Option<String>) -> Result<(), E>;
}

#[async_trait]
pub trait S3BucketNotificationEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_bucket_notification(&self, bucket: &str) -> Result<Vec<String>, E>;
    async fn put_bucket_notification(&self, bucket: &str, configs: Vec<String>) -> Result<(), E>;
}

#[async_trait]
pub trait S3BucketReplicationEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_bucket_replication(&self, bucket: &str) -> Result<Option<BucketReplication>, E>;
    async fn put_bucket_replication(&self, bucket: &str, role: String, rules: Vec<String>) -> Result<(), E>;
    async fn delete_bucket_replication(&self, bucket: &str) -> Result<(), E>;
    async fn get_bucket_replication_metrics(&self, bucket: &str) -> Result<ReplicationMetrics, E>;
    async fn validate_bucket_replication_creds(&self, bucket: &str) -> Result<ReplicationCredsValidation, E>;
}

#[async_trait]
pub trait S3BucketTaggingEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_bucket_tagging(&self, bucket: &str) -> Result<Option<std::collections::HashMap<String, String>>, E>;
    async fn put_bucket_tagging(&self, bucket: &str, tags: std::collections::HashMap<String, String>) -> Result<(), E>;
    async fn delete_bucket_tagging(&self, bucket: &str) -> Result<(), E>;
}

#[async_trait]
pub trait S3BucketWebsiteEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_bucket_website(&self, bucket: &str) -> Result<Option<String>, E>;
    async fn put_bucket_website(&self, bucket: &str, website: String) -> Result<(), E>;
    async fn delete_bucket_website(&self, bucket: &str) -> Result<(), E>;
    async fn set_bucket_cors(&self, bucket: &str, cors: Option<CorsConfiguration>) -> Result<(), E>;
}

#[async_trait]
pub trait S3BucketConfigEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn get_bucket_location(&self, bucket: &str) -> Result<String, E>;
    async fn get_bucket_metadata(&self, bucket: &str) -> Result<BucketMetadataBundle, E>;
    async fn put_bucket_metadata(&self, bucket: &str, metadata: BucketMetadataBundle) -> Result<(), E>;
}

#[async_trait]
pub trait S3BucketEngine<E>:
    S3BucketLifecycleEngine<E>
    + S3BucketEncryptionEngine<E>
    + S3BucketObjectLockEngine<E>
    + S3BucketVersionEngine<E>
    + S3BucketNotificationEngine<E>
    + S3BucketReplicationEngine<E>
    + S3BucketTaggingEngine<E>
    + S3BucketWebsiteEngine<E>
    + S3BucketConfigEngine<E>
    + Send
    + Sync
where
    E: StdError,
{
    async fn make_bucket(&self, bucket: &str, region: Option<&str>, features: BucketFeatures) -> Result<S3Bucket, E>;
    async fn head_bucket(&self, bucket: &str) -> Result<S3Bucket, E>;
    async fn get_bucket(&self, bucket: &str) -> Result<S3Bucket, E>;
    async fn list_buckets(&self) -> Result<Vec<S3Bucket>, E>;
    async fn delete_bucket(&self, bucket: &str, force: bool) -> Result<(), E>;
    async fn list_objects_v1(&self, bucket: &str, options: ListOptions) -> Result<ObjectListPage, E>;
    async fn list_objects_v2(&self, bucket: &str, options: ListOptions) -> Result<ObjectListPage, E>;
    async fn list_object_versions(&self, bucket: &str, options: ListOptions) -> Result<ObjectListPage, E>;
}

#[async_trait]
pub trait S3ObjectEngine<E>:
    S3ObjectTaggingEngine<E> + S3ObjectRetentionEngine<E> + S3ObjectLegalHoldEngine<E> + Send + Sync
where
    E: StdError,
{
    async fn head_object(&self, bucket: &str, key: &str, options: ObjectReadOptions) -> Result<S3Object, E>;
    async fn get_object(&self, bucket: &str, key: &str, options: ObjectReadOptions) -> Result<(S3Object, BoxByteStream), E>;
    async fn put_object(&self, bucket: &str, key: &str, body: BoxByteStream, options: ObjectWriteOptions) -> Result<S3Object, E>;
    async fn copy_object(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str, options: ObjectWriteOptions) -> Result<S3Object, E>;
    async fn delete_object(&self, bucket: &str, key: &str, options: DeleteObjectOptions) -> Result<ObjectVersionRef, E>;
    async fn delete_objects(&self, bucket: &str, keys: Vec<String>, options: DeleteObjectOptions) -> Result<DeleteResult, E>;
}

#[async_trait]
pub trait S3MultipartEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn new_multipart_upload(&self, bucket: &str, key: &str, options: ObjectWriteOptions) -> Result<MultipartUpload, E>;
    async fn put_object_part(&self, bucket: &str, key: &str, upload_id: &str, part_number: u32, body: BoxByteStream) -> Result<UploadedPart, E>;
    async fn copy_object_part(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str, upload_id: &str, part_number: u32) -> Result<UploadedPart, E>;
    async fn list_object_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Vec<UploadedPart>, E>;
    async fn complete_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str, completed: CompleteMultipartInput) -> Result<S3Object, E>;
    async fn abort_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<(), E>;
    async fn list_multipart_uploads(&self, bucket: &str, options: ListOptions) -> Result<Vec<MultipartUpload>, E>;
}

pub trait S3Engine<E>: S3BucketEngine<E> + S3ObjectEngine<E> + S3MultipartEngine<E> + Send + Sync
where
    E: StdError,
{
}

impl<T, E> S3Engine<E> for T
where
    T: S3BucketEngine<E> + S3ObjectEngine<E> + S3MultipartEngine<E> + Send + Sync,
    E: StdError,
{
}
