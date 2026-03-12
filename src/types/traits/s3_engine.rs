use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::traits::StdError;

#[async_trait]
pub trait S3BucketEngine<E>:
    S3BucketLifecycleEngine<Error = E>
    + S3BucketEncryptionEngine<Error = E>
    + S3BucketObjectLockEngine<Error = E>
    + S3BucketVersionEngine<Error = E>
    + S3BucketNotificationEngine<Error = E>
    + S3BucketReplicationEngine<Error = E>
    + S3BucketTaggingEngine<Error = E>
    + S3BucketWebsiteEngine<Error = E>
    + S3BucketConfigEngine<Error = E>
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
    S3ObjectTaggingEngine<Error = E>
    + S3ObjectRetentionEngine<Error = E>
    + S3ObjectLegalHoldEngine<Error = E>
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
pub trait S3ObjectTaggingEngine {
    type Error: StdError;

    async fn get_object_tagging(&self, bucket: &str, key: &str) -> Result<TagMap, Self::Error>;
    async fn put_object_tagging(&self, bucket: &str, key: &str, tags: TagMap) -> Result<(), Self::Error>;
    async fn delete_object_tagging(&self, bucket: &str, key: &str) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait S3ObjectRetentionEngine {
    type Error: StdError;

    async fn get_object_retention(&self, bucket: &str, key: &str) -> Result<Option<ObjectRetention>, Self::Error>;
    async fn put_object_retention(&self, bucket: &str, key: &str, retention: ObjectRetention) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait S3ObjectLegalHoldEngine {
    type Error: StdError;

    async fn get_object_legal_hold(&self, bucket: &str, key: &str) -> Result<Option<ObjectLegalHold>, Self::Error>;
    async fn put_object_legal_hold(&self, bucket: &str, key: &str, legal_hold: ObjectLegalHold) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait S3MultipartEngine {
    type Error: StdError;

    async fn new_multipart_upload(&self, bucket: &str, key: &str, options: ObjectWriteOptions) -> Result<MultipartUpload, Self::Error>;
    async fn put_object_part(&self, bucket: &str, key: &str, upload_id: &str, part_number: u32, body: BoxByteStream) -> Result<UploadedPart, Self::Error>;
    async fn copy_object_part(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str, upload_id: &str, part_number: u32) -> Result<UploadedPart, Self::Error>;
    async fn list_object_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Vec<UploadedPart>, Self::Error>;
    async fn complete_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str, completed: CompleteMultipartInput) -> Result<S3Object, Self::Error>;
    async fn abort_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<(), Self::Error>;
    async fn list_multipart_uploads(&self, bucket: &str, options: ListOptions) -> Result<Vec<MultipartUpload>, Self::Error>;
}

#[async_trait]
pub trait S3BucketLifecycleEngine {
    type Error: StdError;

    async fn get_bucket_lifecycle(&self, bucket: &str) -> Result<Vec<String>, Self::Error>;
    async fn put_bucket_lifecycle(&self, bucket: &str, rules: Vec<String>) -> Result<(), Self::Error>;
    async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait S3BucketEncryptionEngine {
    type Error: StdError;

    async fn get_bucket_encryption(&self, bucket: &str) -> Result<Option<BucketEncryption>, Self::Error>;
    async fn put_bucket_encryption(&self, bucket: &str, algorithm: String, key_id: Option<String>) -> Result<(), Self::Error>;
    async fn delete_bucket_encryption(&self, bucket: &str) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait S3BucketObjectLockEngine {
    type Error: StdError;

    async fn get_bucket_object_lock_config(&self, bucket: &str) -> Result<Option<BucketObjectLockConfig>, Self::Error>;
    async fn put_bucket_object_lock_config(&self, bucket: &str, enabled: bool, mode: Option<String>, days: Option<u32>, years: Option<u32>) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait S3BucketVersionEngine {
    type Error: StdError;

    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<BucketVersioning>, Self::Error>;
    async fn put_bucket_versioning(&self, bucket: &str, status: String, mfa_delete: Option<String>) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait S3BucketNotificationEngine {
    type Error: StdError;

    async fn get_bucket_notification(&self, bucket: &str) -> Result<Vec<String>, Self::Error>;
    async fn put_bucket_notification(&self, bucket: &str, configs: Vec<String>) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait S3BucketReplicationEngine {
    type Error: StdError;

    async fn get_bucket_replication(&self, bucket: &str) -> Result<Option<BucketReplication>, Self::Error>;
    async fn put_bucket_replication(&self, bucket: &str, role: String, rules: Vec<String>) -> Result<(), Self::Error>;
    async fn delete_bucket_replication(&self, bucket: &str) -> Result<(), Self::Error>;
    async fn get_bucket_replication_metrics(&self, bucket: &str) -> Result<ReplicationMetrics, Self::Error>;
    async fn validate_bucket_replication_creds(&self, bucket: &str) -> Result<ReplicationCredsValidation, Self::Error>;
}

#[async_trait]
pub trait S3BucketTaggingEngine {
    type Error: StdError;

    async fn get_bucket_tagging(&self, bucket: &str) -> Result<Option<std::collections::HashMap<String, String>>, Self::Error>;
    async fn put_bucket_tagging(&self, bucket: &str, tags: std::collections::HashMap<String, String>) -> Result<(), Self::Error>;
    async fn delete_bucket_tagging(&self, bucket: &str) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait S3BucketWebsiteEngine {
    type Error: StdError;

    async fn get_bucket_website(&self, bucket: &str) -> Result<Option<String>, Self::Error>;
    async fn put_bucket_website(&self, bucket: &str, website: String) -> Result<(), Self::Error>;
    async fn delete_bucket_website(&self, bucket: &str) -> Result<(), Self::Error>;
    async fn set_bucket_cors(&self, bucket: &str, cors: Option<CorsConfiguration>) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait S3BucketConfigEngine {
    type Error: StdError;

    async fn get_bucket_location(&self, bucket: &str) -> Result<String, Self::Error>;
    async fn get_bucket_metadata(&self, bucket: &str) -> Result<BucketMetadataBundle, Self::Error>;
    async fn put_bucket_metadata(&self, bucket: &str, metadata: BucketMetadataBundle) -> Result<(), Self::Error>;
}

pub trait S3Engine<E>: S3BucketEngine<E> + S3ObjectEngine<E> + S3MultipartEngine<Error = E>
where
    E: StdError,
{
}

impl<T, E> S3Engine<E> for T
where
    T: S3BucketEngine<E> + S3ObjectEngine<E> + S3MultipartEngine<Error = E>,
    E: StdError,
{
}
