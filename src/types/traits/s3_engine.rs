use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::errors::S3EngineError;

#[async_trait]
pub trait S3BucketEngine {
    async fn make_bucket(&self, bucket: &str, region: Option<&str>, features: BucketFeatures) -> Result<S3Bucket, S3EngineError>;
    async fn head_bucket(&self, bucket: &str) -> Result<S3Bucket, S3EngineError>;
    async fn get_bucket(&self, bucket: &str) -> Result<S3Bucket, S3EngineError>;
    async fn list_buckets(&self) -> Result<Vec<S3Bucket>, S3EngineError>;
    async fn delete_bucket(&self, bucket: &str, force: bool) -> Result<(), S3EngineError>;
    async fn list_objects_v1(&self, bucket: &str, options: ListOptions) -> Result<ObjectListPage, S3EngineError>;
    async fn list_objects_v2(&self, bucket: &str, options: ListOptions) -> Result<ObjectListPage, S3EngineError>;
    async fn list_object_versions(&self, bucket: &str, options: ListOptions) -> Result<ObjectListPage, S3EngineError>;
}

#[async_trait]
pub trait S3ObjectEngine {
    async fn head_object(&self, bucket: &str, key: &str, options: ObjectReadOptions) -> Result<S3Object, S3EngineError>;
    async fn get_object(&self, bucket: &str, key: &str, options: ObjectReadOptions) -> Result<(S3Object, BoxByteStream), S3EngineError>;
    async fn put_object(&self, bucket: &str, key: &str, body: BoxByteStream, options: ObjectWriteOptions) -> Result<S3Object, S3EngineError>;
    async fn copy_object(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str, options: ObjectWriteOptions) -> Result<S3Object, S3EngineError>;
    async fn delete_object(&self, bucket: &str, key: &str, options: DeleteObjectOptions) -> Result<ObjectVersionRef, S3EngineError>;
    async fn delete_objects(&self, bucket: &str, keys: Vec<String>, options: DeleteObjectOptions) -> Result<DeleteResult, S3EngineError>;
    async fn get_object_tagging(&self, bucket: &str, key: &str) -> Result<TagMap, S3EngineError>;
    async fn put_object_tagging(&self, bucket: &str, key: &str, tags: TagMap) -> Result<(), S3EngineError>;
    async fn delete_object_tagging(&self, bucket: &str, key: &str) -> Result<(), S3EngineError>;
    async fn get_object_retention(&self, bucket: &str, key: &str) -> Result<Option<ObjectRetention>, S3EngineError>;
    async fn put_object_retention(&self, bucket: &str, key: &str, retention: ObjectRetention) -> Result<(), S3EngineError>;
    async fn get_object_legal_hold(&self, bucket: &str, key: &str) -> Result<Option<ObjectLegalHold>, S3EngineError>;
    async fn put_object_legal_hold(&self, bucket: &str, key: &str, legal_hold: ObjectLegalHold) -> Result<(), S3EngineError>;
}

#[async_trait]
pub trait S3MultipartEngine {
    async fn new_multipart_upload(&self, bucket: &str, key: &str, options: ObjectWriteOptions) -> Result<MultipartUpload, S3EngineError>;
    async fn put_object_part(&self, bucket: &str, key: &str, upload_id: &str, part_number: u32, body: BoxByteStream) -> Result<UploadedPart, S3EngineError>;
    async fn copy_object_part(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str, upload_id: &str, part_number: u32) -> Result<UploadedPart, S3EngineError>;
    async fn list_object_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Vec<UploadedPart>, S3EngineError>;
    async fn complete_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str, completed: CompleteMultipartInput) -> Result<S3Object, S3EngineError>;
    async fn abort_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<(), S3EngineError>;
    async fn list_multipart_uploads(&self, bucket: &str, options: ListOptions) -> Result<Vec<MultipartUpload>, S3EngineError>;
}

#[async_trait]
pub trait S3BucketConfigEngine {
    async fn get_bucket_location(&self, bucket: &str) -> Result<String, S3EngineError>;
    async fn get_bucket_lifecycle(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineError>;
    async fn put_bucket_lifecycle(&self, bucket: &str, lifecycle_xml: String) -> Result<(), S3EngineError>;
    async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<(), S3EngineError>;
    async fn get_bucket_encryption(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineError>;
    async fn put_bucket_encryption(&self, bucket: &str, encryption_xml: String) -> Result<(), S3EngineError>;
    async fn delete_bucket_encryption(&self, bucket: &str) -> Result<(), S3EngineError>;
    async fn get_bucket_object_lock_config(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineError>;
    async fn put_bucket_object_lock_config(&self, bucket: &str, object_lock_xml: String) -> Result<(), S3EngineError>;
    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineError>;
    async fn put_bucket_versioning(&self, bucket: &str, versioning_xml: String) -> Result<(), S3EngineError>;
    async fn get_bucket_notification(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineError>;
    async fn put_bucket_notification(&self, bucket: &str, notification_xml: String) -> Result<(), S3EngineError>;
    async fn get_bucket_replication(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineError>;
    async fn put_bucket_replication(&self, bucket: &str, replication_xml: String) -> Result<(), S3EngineError>;
    async fn delete_bucket_replication(&self, bucket: &str) -> Result<(), S3EngineError>;
    async fn get_bucket_tagging(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineError>;
    async fn put_bucket_tagging(&self, bucket: &str, tagging_xml: String) -> Result<(), S3EngineError>;
    async fn delete_bucket_tagging(&self, bucket: &str) -> Result<(), S3EngineError>;
    async fn get_bucket_metadata(&self, bucket: &str) -> Result<BucketMetadataBundle, S3EngineError>;
    async fn put_bucket_metadata(&self, bucket: &str, metadata: BucketMetadataBundle) -> Result<(), S3EngineError>;
    async fn get_bucket_replication_metrics(&self, bucket: &str) -> Result<ReplicationMetrics, S3EngineError>;
    async fn validate_bucket_replication_creds(&self, bucket: &str) -> Result<ReplicationCredsValidation, S3EngineError>;
}

pub trait S3Engine:
    S3BucketEngine + S3ObjectEngine + S3MultipartEngine + S3BucketConfigEngine
{
}

impl<T> S3Engine for T
where
    T: S3BucketEngine + S3ObjectEngine + S3MultipartEngine + S3BucketConfigEngine,
{
}
