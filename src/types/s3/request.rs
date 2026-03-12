use crate::types::s3::core::BoxByteStream;
use crate::types::s3::core::ObjectAttribute;
use crate::types::s3::core::{
    BucketEncryption, BucketObjectLockConfig, BucketReplication, BucketVersioning, BucketWebsite,
    CompleteMultipartInput, CorsConfiguration, ObjectLegalHold, ObjectRetention,
};
use crate::types::s3::xml::{
    AccessControlPolicyInput, LifecycleRuleInput, NotificationConfigInput, RestoreObjectInput,
    SelectObjectContentInput,
};
use std::collections::HashMap;

pub type HeaderMap = HashMap<String, String>;

pub struct RequestContext {
    pub host: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

pub struct BucketRef {
    pub bucket: String,
}

pub struct ObjectRef {
    pub bucket: String,
    pub object: String,
}

pub struct MultipartSelector {
    pub upload_id: String,
    pub part_number: Option<u32>,
}

pub struct ListQuery {
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<u32>,
    pub continuation_token: Option<String>,
    pub start_after: Option<String>,
    pub marker: Option<String>,
    pub version_id_marker: Option<String>,
    pub key_marker: Option<String>,
}

pub struct PostPolicyForm {
    pub fields: HashMap<String, String>,
    pub payload: BoxByteStream,
}

pub struct DeleteObjectsInput {
    pub quiet: bool,
    pub keys: Vec<String>,
}

pub struct EventFilter {
    pub events: Vec<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

pub struct S3RequestEnvelope {
    pub context: Option<RequestContext>,
    pub headers: HeaderMap,
}

pub struct HeadObjectRequest {
    pub object: ObjectRef,
    pub range: Option<String>,
    pub version_id: Option<String>,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
}

pub struct GetObjectAttributesRequest {
    pub object: ObjectRef,
    pub attributes: Vec<ObjectAttribute>,
}

pub struct CopyObjectPartRequest {
    pub object: ObjectRef,
    pub multipart: MultipartSelector,
    pub copy_source: String,
}

pub struct PutObjectPartRequest {
    pub object: ObjectRef,
    pub multipart: MultipartSelector,
    pub body: crate::types::s3::core::BoxByteStream,
    pub checksum: Option<String>,
    pub content_md5: Option<String>,
    pub content_encoding: Option<String>,
    pub amz_content_sha256: Option<String>,
    pub decoded_content_length: Option<String>,
    pub amz_trailer: Option<String>,
}

pub struct ListObjectPartsRequest {
    pub object: ObjectRef,
    pub upload_id: String,
}

pub struct CompleteMultipartUploadRequest {
    pub object: ObjectRef,
    pub upload_id: String,
    pub completed: CompleteMultipartInput,
}

pub struct NewMultipartUploadRequest {
    pub object: ObjectRef,
}

pub struct AbortMultipartUploadRequest {
    pub object: ObjectRef,
    pub upload_id: String,
}

pub struct GetObjectAclRequest {
    pub object: ObjectRef,
}

pub struct PutObjectAclRequest {
    pub object: ObjectRef,
    pub acl: Option<AccessControlPolicyInput>,
}

pub struct GetObjectTaggingRequest {
    pub object: ObjectRef,
}

pub struct PutObjectTaggingRequest {
    pub object: ObjectRef,
    pub tags: HashMap<String, String>,
}

pub struct DeleteObjectTaggingRequest {
    pub object: ObjectRef,
}

pub struct SelectObjectContentRequest {
    pub object: ObjectRef,
    pub select_type: u8,
    pub input: SelectObjectContentInput,
}

pub struct GetObjectRetentionRequest {
    pub bucket: BucketRef,
    pub object: ObjectRef,
}

pub struct GetObjectLegalHoldRequest {
    pub bucket: BucketRef,
    pub object: ObjectRef,
}

pub struct GetObjectLambdaRequest {
    pub object: ObjectRef,
    pub lambda_arn: String,
    pub range: Option<String>,
    pub version_id: Option<String>,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
}

pub struct GetObjectRequest {
    pub object: ObjectRef,
    pub range: Option<String>,
    pub version_id: Option<String>,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
}

pub struct CopyObjectRequest {
    pub object: ObjectRef,
    pub copy_source: String,
    pub copy_source_version_id: Option<String>,
    pub metadata_directive: Option<String>,
    pub tagging_directive: Option<String>,
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub storage_class: Option<String>,
    pub user_metadata: std::collections::HashMap<String, String>,
}

pub struct PutObjectRetentionRequest {
    pub bucket: BucketRef,
    pub object: ObjectRef,
    pub retention: ObjectRetention,
}

pub struct PutObjectLegalHoldRequest {
    pub bucket: BucketRef,
    pub object: ObjectRef,
    pub legal_hold: ObjectLegalHold,
}

pub struct PutObjectExtractRequest {
    pub object: ObjectRef,
    pub body: BoxByteStream,
}

pub struct AppendObjectRejectedRequest {
    pub object: ObjectRef,
    pub write_offset_bytes: String,
    pub body: BoxByteStream,
}

pub struct PutObjectRequest {
    pub object: ObjectRef,
    pub body: BoxByteStream,
    pub content_type: Option<String>,
    pub content_md5: Option<String>,
    pub checksum_sha256: Option<String>,
    pub checksum_sha1: Option<String>,
    pub checksum_crc32: Option<String>,
    pub checksum_crc32c: Option<String>,
    pub content_length: Option<u64>,
    pub content_encoding: Option<String>,
    pub amz_content_sha256: Option<String>,
    pub decoded_content_length: Option<String>,
    pub amz_trailer: Option<String>,
    pub sse: Option<String>,
    pub sse_customer_algorithm: Option<String>,
    pub sse_customer_key: Option<String>,
    pub sse_customer_key_md5: Option<String>,
    pub sse_kms_key_id: Option<String>,
    pub sse_context: Option<String>,
    pub user_metadata: std::collections::HashMap<String, String>,
}

pub struct DeleteObjectRequest {
    pub object: ObjectRef,
    pub version_id: Option<String>,
}

pub struct PostRestoreObjectRequest {
    pub object: ObjectRef,
    pub restore: RestoreObjectInput,
}

pub struct GetBucketLocationRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketPolicyRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketLifecycleRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketEncryptionRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketObjectLockConfigRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketReplicationConfigRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketVersioningRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketNotificationRequest {
    pub bucket: BucketRef,
}

pub struct ListenBucketNotificationRequest {
    pub bucket: BucketRef,
    pub filter: EventFilter,
}

pub struct ResetBucketReplicationStatusRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketAclRequest {
    pub bucket: BucketRef,
}

pub struct PutBucketAclRequest {
    pub bucket: BucketRef,
    pub acl: Option<AccessControlPolicyInput>,
}

pub struct GetBucketCorsRequest {
    pub bucket: BucketRef,
}

pub struct PutBucketCorsRequest {
    pub bucket: BucketRef,
    pub cors: CorsConfiguration,
}

pub struct DeleteBucketCorsRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketWebsiteRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketAccelerateRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketRequestPaymentRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketLoggingRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketTaggingRequest {
    pub bucket: BucketRef,
}

pub struct PutBucketWebsiteRequest {
    pub bucket: BucketRef,
    pub website: BucketWebsite,
}

pub struct DeleteBucketWebsiteRequest {
    pub bucket: BucketRef,
}

pub struct DeleteBucketTaggingRequest {
    pub bucket: BucketRef,
}

pub struct ListMultipartUploadsRequest {
    pub bucket: BucketRef,
    pub query: ListQuery,
}

pub struct ListObjectsV2MRequest {
    pub bucket: BucketRef,
    pub query: ListQuery,
    pub metadata: bool,
}

pub struct ListObjectsV2Request {
    pub bucket: BucketRef,
    pub query: ListQuery,
}

pub struct ListObjectVersionsMRequest {
    pub bucket: BucketRef,
    pub query: ListQuery,
    pub metadata: bool,
}

pub struct ListObjectVersionsRequest {
    pub bucket: BucketRef,
    pub query: ListQuery,
}

pub struct GetBucketPolicyStatusRequest {
    pub bucket: BucketRef,
}

pub struct PutBucketLifecycleRequest {
    pub bucket: BucketRef,
    pub rules: Vec<LifecycleRuleInput>,
}

pub struct PutBucketReplicationConfigRequest {
    pub bucket: BucketRef,
    pub replication: BucketReplication,
}

pub struct PutBucketEncryptionRequest {
    pub bucket: BucketRef,
    pub encryption: BucketEncryption,
}

pub struct PutBucketPolicyRequest {
    pub bucket: BucketRef,
    pub json: String,
}

pub struct PutBucketObjectLockConfigRequest {
    pub bucket: BucketRef,
    pub config: BucketObjectLockConfig,
}

pub struct PutBucketTaggingRequest {
    pub bucket: BucketRef,
    pub tags: HashMap<String, String>,
}

pub struct PutBucketVersioningRequest {
    pub bucket: BucketRef,
    pub versioning: BucketVersioning,
}

pub struct PutBucketNotificationRequest {
    pub bucket: BucketRef,
    pub configs: Vec<NotificationConfigInput>,
}

pub struct ResetBucketReplicationStartRequest {
    pub bucket: BucketRef,
}

pub struct PutBucketRequest {
    pub bucket: BucketRef,
    pub region: Option<String>,
}

pub struct HeadBucketRequest {
    pub bucket: BucketRef,
}

pub struct PostPolicyRequest {
    pub bucket: BucketRef,
    pub form: PostPolicyForm,
}

pub struct DeleteMultipleObjectsRequest {
    pub bucket: BucketRef,
    pub payload: DeleteObjectsInput,
}

pub struct DeleteBucketPolicyRequest {
    pub bucket: BucketRef,
}

pub struct DeleteBucketReplicationRequest {
    pub bucket: BucketRef,
}

pub struct DeleteBucketLifecycleRequest {
    pub bucket: BucketRef,
}

pub struct DeleteBucketEncryptionRequest {
    pub bucket: BucketRef,
}

pub struct DeleteBucketRequest {
    pub bucket: BucketRef,
}

pub struct GetBucketReplicationMetricsV2Request {
    pub bucket: BucketRef,
}

pub struct GetBucketReplicationMetricsRequest {
    pub bucket: BucketRef,
}

pub struct ValidateBucketReplicationCredsRequest {
    pub bucket: BucketRef,
}

pub struct ListObjectsV1Request {
    pub bucket: BucketRef,
    pub query: ListQuery,
}

pub struct RootListenNotificationRequest {
    pub filter: EventFilter,
}

pub struct ListBucketsRequest;

pub struct ListBucketsDoubleSlashRequest;

pub struct RejectedObjectTorrentRequest {
    pub object: ObjectRef,
    pub method: String,
}

pub struct RejectedObjectAclDeleteRequest {
    pub object: ObjectRef,
}

pub struct RejectedBucketApiRequest {
    pub bucket: BucketRef,
    pub api: String,
    pub method: String,
}

pub enum S3Request {
    HeadObject(HeadObjectRequest),
    GetObjectAttributes(GetObjectAttributesRequest),
    CopyObjectPart(CopyObjectPartRequest),
    PutObjectPart(PutObjectPartRequest),
    ListObjectParts(ListObjectPartsRequest),
    CompleteMultipartUpload(CompleteMultipartUploadRequest),
    NewMultipartUpload(NewMultipartUploadRequest),
    AbortMultipartUpload(AbortMultipartUploadRequest),
    GetObjectAcl(GetObjectAclRequest),
    PutObjectAcl(PutObjectAclRequest),
    GetObjectTagging(GetObjectTaggingRequest),
    PutObjectTagging(PutObjectTaggingRequest),
    DeleteObjectTagging(DeleteObjectTaggingRequest),
    SelectObjectContent(SelectObjectContentRequest),
    GetObjectRetention(GetObjectRetentionRequest),
    GetObjectLegalHold(GetObjectLegalHoldRequest),
    GetObjectLambda(GetObjectLambdaRequest),
    GetObject(GetObjectRequest),
    CopyObject(CopyObjectRequest),
    PutObjectRetention(PutObjectRetentionRequest),
    PutObjectLegalHold(PutObjectLegalHoldRequest),
    PutObjectExtract(PutObjectExtractRequest),
    AppendObjectRejected(AppendObjectRejectedRequest),
    PutObject(PutObjectRequest),
    DeleteObject(DeleteObjectRequest),
    PostRestoreObject(PostRestoreObjectRequest),
    GetBucketLocation(GetBucketLocationRequest),
    GetBucketPolicy(GetBucketPolicyRequest),
    GetBucketLifecycle(GetBucketLifecycleRequest),
    GetBucketEncryption(GetBucketEncryptionRequest),
    GetBucketObjectLockConfig(GetBucketObjectLockConfigRequest),
    GetBucketReplicationConfig(GetBucketReplicationConfigRequest),
    GetBucketVersioning(GetBucketVersioningRequest),
    GetBucketNotification(GetBucketNotificationRequest),
    ListenBucketNotification(ListenBucketNotificationRequest),
    ResetBucketReplicationStatus(ResetBucketReplicationStatusRequest),
    GetBucketAcl(GetBucketAclRequest),
    PutBucketAcl(PutBucketAclRequest),
    GetBucketCors(GetBucketCorsRequest),
    PutBucketCors(PutBucketCorsRequest),
    DeleteBucketCors(DeleteBucketCorsRequest),
    GetBucketWebsite(GetBucketWebsiteRequest),
    PutBucketWebsite(PutBucketWebsiteRequest),
    GetBucketAccelerate(GetBucketAccelerateRequest),
    GetBucketRequestPayment(GetBucketRequestPaymentRequest),
    GetBucketLogging(GetBucketLoggingRequest),
    GetBucketTagging(GetBucketTaggingRequest),
    DeleteBucketWebsite(DeleteBucketWebsiteRequest),
    DeleteBucketTagging(DeleteBucketTaggingRequest),
    ListMultipartUploads(ListMultipartUploadsRequest),
    ListObjectsV2M(ListObjectsV2MRequest),
    ListObjectsV2(ListObjectsV2Request),
    ListObjectVersionsM(ListObjectVersionsMRequest),
    ListObjectVersions(ListObjectVersionsRequest),
    GetBucketPolicyStatus(GetBucketPolicyStatusRequest),
    PutBucketLifecycle(PutBucketLifecycleRequest),
    PutBucketReplicationConfig(PutBucketReplicationConfigRequest),
    PutBucketEncryption(PutBucketEncryptionRequest),
    PutBucketPolicy(PutBucketPolicyRequest),
    PutBucketObjectLockConfig(PutBucketObjectLockConfigRequest),
    PutBucketTagging(PutBucketTaggingRequest),
    PutBucketVersioning(PutBucketVersioningRequest),
    PutBucketNotification(PutBucketNotificationRequest),
    ResetBucketReplicationStart(ResetBucketReplicationStartRequest),
    PutBucket(PutBucketRequest),
    HeadBucket(HeadBucketRequest),
    PostPolicy(PostPolicyRequest),
    DeleteMultipleObjects(DeleteMultipleObjectsRequest),
    DeleteBucketPolicy(DeleteBucketPolicyRequest),
    DeleteBucketReplication(DeleteBucketReplicationRequest),
    DeleteBucketLifecycle(DeleteBucketLifecycleRequest),
    DeleteBucketEncryption(DeleteBucketEncryptionRequest),
    DeleteBucket(DeleteBucketRequest),
    GetBucketReplicationMetricsV2(GetBucketReplicationMetricsV2Request),
    GetBucketReplicationMetrics(GetBucketReplicationMetricsRequest),
    ValidateBucketReplicationCreds(ValidateBucketReplicationCredsRequest),
    ListObjectsV1(ListObjectsV1Request),
    RootListenNotification(RootListenNotificationRequest),
    ListBuckets(ListBucketsRequest),
    ListBucketsDoubleSlash(ListBucketsDoubleSlashRequest),
    RejectedObjectTorrent(RejectedObjectTorrentRequest),
    RejectedObjectAclDelete(RejectedObjectAclDeleteRequest),
    RejectedBucketApi(RejectedBucketApiRequest),
}
