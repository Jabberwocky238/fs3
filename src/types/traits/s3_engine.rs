use async_trait::async_trait;

use crate::types::s3::core::*;

pub trait S3EngineError: std::fmt::Display + Send + Sync + 'static {}
impl<T: std::fmt::Display + Send + Sync + 'static> S3EngineError for T {}

#[async_trait]
pub trait S3BucketEngine<E: S3EngineError> {
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
pub trait S3ObjectEngine<E: S3EngineError> {
    async fn head_object(&self, bucket: &str, key: &str, options: ObjectReadOptions) -> Result<S3Object, E>;
    async fn get_object(&self, bucket: &str, key: &str, options: ObjectReadOptions) -> Result<(S3Object, BoxByteStream), E>;
    async fn put_object(&self, bucket: &str, key: &str, body: BoxByteStream, options: ObjectWriteOptions) -> Result<S3Object, E>;
    async fn copy_object(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str, options: ObjectWriteOptions) -> Result<S3Object, E>;
    async fn delete_object(&self, bucket: &str, key: &str, options: DeleteObjectOptions) -> Result<ObjectVersionRef, E>;
    async fn delete_objects(&self, bucket: &str, keys: Vec<String>, options: DeleteObjectOptions) -> Result<DeleteResult, E>;
    async fn get_object_tagging(&self, bucket: &str, key: &str) -> Result<TagMap, E>;
    async fn put_object_tagging(&self, bucket: &str, key: &str, tags: TagMap) -> Result<(), E>;
    async fn delete_object_tagging(&self, bucket: &str, key: &str) -> Result<(), E>;
    async fn get_object_retention(&self, bucket: &str, key: &str) -> Result<Option<ObjectRetention>, E>;
    async fn put_object_retention(&self, bucket: &str, key: &str, retention: ObjectRetention) -> Result<(), E>;
    async fn get_object_legal_hold(&self, bucket: &str, key: &str) -> Result<Option<ObjectLegalHold>, E>;
    async fn put_object_legal_hold(&self, bucket: &str, key: &str, legal_hold: ObjectLegalHold) -> Result<(), E>;
}

#[async_trait]
pub trait S3MultipartEngine<E: S3EngineError> {
    async fn new_multipart_upload(&self, bucket: &str, key: &str, options: ObjectWriteOptions) -> Result<MultipartUpload, E>;
    async fn put_object_part(&self, bucket: &str, key: &str, upload_id: &str, part_number: u32, body: BoxByteStream) -> Result<UploadedPart, E>;
    async fn copy_object_part(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str, upload_id: &str, part_number: u32) -> Result<UploadedPart, E>;
    async fn list_object_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Vec<UploadedPart>, E>;
    async fn complete_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str, completed: CompleteMultipartInput) -> Result<S3Object, E>;
    async fn abort_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<(), E>;
    async fn list_multipart_uploads(&self, bucket: &str, options: ListOptions) -> Result<Vec<MultipartUpload>, E>;
}

#[async_trait]
pub trait S3BucketConfigEngine<E: S3EngineError> {
    async fn get_bucket_location(&self, bucket: &str) -> Result<String, E>;
    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<TimedDocument>, E>;
    async fn put_bucket_policy(&self, bucket: &str, policy_json: String) -> Result<(), E>;
    async fn delete_bucket_policy(&self, bucket: &str) -> Result<(), E>;
    async fn get_bucket_policy_status(&self, bucket: &str) -> Result<BucketPolicyStatus, E>;
    async fn get_bucket_lifecycle(&self, bucket: &str) -> Result<Option<TimedDocument>, E>;
    async fn put_bucket_lifecycle(&self, bucket: &str, lifecycle_xml: String) -> Result<(), E>;
    async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<(), E>;
    async fn get_bucket_encryption(&self, bucket: &str) -> Result<Option<TimedDocument>, E>;
    async fn put_bucket_encryption(&self, bucket: &str, encryption_xml: String) -> Result<(), E>;
    async fn delete_bucket_encryption(&self, bucket: &str) -> Result<(), E>;
    async fn get_bucket_object_lock_config(&self, bucket: &str) -> Result<Option<TimedDocument>, E>;
    async fn put_bucket_object_lock_config(&self, bucket: &str, object_lock_xml: String) -> Result<(), E>;
    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<TimedDocument>, E>;
    async fn put_bucket_versioning(&self, bucket: &str, versioning_xml: String) -> Result<(), E>;
    async fn get_bucket_notification(&self, bucket: &str) -> Result<Option<TimedDocument>, E>;
    async fn put_bucket_notification(&self, bucket: &str, notification_xml: String) -> Result<(), E>;
    async fn get_bucket_replication(&self, bucket: &str) -> Result<Option<TimedDocument>, E>;
    async fn put_bucket_replication(&self, bucket: &str, replication_xml: String) -> Result<(), E>;
    async fn delete_bucket_replication(&self, bucket: &str) -> Result<(), E>;
    async fn get_bucket_tagging(&self, bucket: &str) -> Result<Option<TimedDocument>, E>;
    async fn put_bucket_tagging(&self, bucket: &str, tagging_xml: String) -> Result<(), E>;
    async fn delete_bucket_tagging(&self, bucket: &str) -> Result<(), E>;
    async fn get_bucket_metadata(&self, bucket: &str) -> Result<BucketMetadataBundle, E>;
    async fn put_bucket_metadata(&self, bucket: &str, metadata: BucketMetadataBundle) -> Result<(), E>;
    async fn get_bucket_replication_metrics(&self, bucket: &str) -> Result<ReplicationMetrics, E>;
    async fn validate_bucket_replication_creds(&self, bucket: &str) -> Result<ReplicationCredsValidation, E>;
}

pub trait S3Engine<E: S3EngineError>:
    S3BucketEngine<E> + S3ObjectEngine<E> + S3MultipartEngine<E> + S3BucketConfigEngine<E>
{
}

impl<T, E> S3Engine<E> for T
where
    E: S3EngineError,
    T: S3BucketEngine<E> + S3ObjectEngine<E> + S3MultipartEngine<E> + S3BucketConfigEngine<E>,
{
}
