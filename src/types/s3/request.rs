use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::types::s3::core::ObjectAttribute;

pub type HeaderMap = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RequestContext {
    pub host: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BucketRef {
    pub bucket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ObjectRef {
    pub bucket: String,
    pub object: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MultipartSelector {
    #[serde(rename = "uploadId")]
    pub upload_id: String,
    #[serde(rename = "partNumber")]
    pub part_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListQuery {
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    #[serde(rename = "max-keys")]
    pub max_keys: Option<u32>,
    #[serde(rename = "continuation-token")]
    pub continuation_token: Option<String>,
    #[serde(rename = "start-after")]
    pub start_after: Option<String>,
    pub marker: Option<String>,
    #[serde(rename = "version-id-marker")]
    pub version_id_marker: Option<String>,
    #[serde(rename = "key-marker")]
    pub key_marker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PostPolicyForm {
    pub fields: HashMap<String, String>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteObjectsInput {
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RawDocument {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EventFilter {
    pub events: Vec<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct S3RequestEnvelope {
    pub context: Option<RequestContext>,
    pub headers: HeaderMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HeadObjectRequest {
    pub object: ObjectRef,
    pub range: Option<String>,
    #[serde(rename = "versionId")]
    pub version_id: Option<String>,
    #[serde(rename = "If-Match")]
    pub if_match: Option<String>,
    #[serde(rename = "If-None-Match")]
    pub if_none_match: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectAttributesRequest {
    pub object: ObjectRef,
    pub attributes: Vec<ObjectAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CopyObjectPartRequest {
    pub object: ObjectRef,
    pub multipart: MultipartSelector,
    #[serde(rename = "x-amz-copy-source")]
    pub copy_source: String,
}

pub struct PutObjectPartRequest {
    pub object: ObjectRef,
    pub multipart: MultipartSelector,
    pub body: crate::types::s3::core::BoxByteStream,
    pub checksum: Option<String>,
}

impl std::fmt::Debug for PutObjectPartRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PutObjectPartRequest").field("object", &self.object).field("body", &"<stream>").finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectPartsRequest {
    pub object: ObjectRef,
    #[serde(rename = "uploadId")]
    pub upload_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CompleteMultipartUploadRequest {
    pub object: ObjectRef,
    #[serde(rename = "uploadId")]
    pub upload_id: String,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NewMultipartUploadRequest {
    pub object: ObjectRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AbortMultipartUploadRequest {
    pub object: ObjectRef,
    #[serde(rename = "uploadId")]
    pub upload_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectAclRequest {
    pub object: ObjectRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectAclRequest {
    pub object: ObjectRef,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectTaggingRequest {
    pub object: ObjectRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectTaggingRequest {
    pub object: ObjectRef,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteObjectTaggingRequest {
    pub object: ObjectRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SelectObjectContentRequest {
    pub object: ObjectRef,
    #[serde(rename = "select-type")]
    pub select_type: u8,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectRetentionRequest {
    pub bucket: BucketRef,
    pub object: ObjectRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectLegalHoldRequest {
    pub bucket: BucketRef,
    pub object: ObjectRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectLambdaRequest {
    pub object: ObjectRef,
    #[serde(rename = "lambdaArn")]
    pub lambda_arn: String,
    pub range: Option<String>,
    #[serde(rename = "versionId")]
    pub version_id: Option<String>,
    #[serde(rename = "If-Match")]
    pub if_match: Option<String>,
    #[serde(rename = "If-None-Match")]
    pub if_none_match: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectRequest {
    pub object: ObjectRef,
    pub range: Option<String>,
    #[serde(rename = "versionId")]
    pub version_id: Option<String>,
    #[serde(rename = "If-Match")]
    pub if_match: Option<String>,
    #[serde(rename = "If-None-Match")]
    pub if_none_match: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CopyObjectRequest {
    pub object: ObjectRef,
    #[serde(rename = "x-amz-copy-source")]
    pub copy_source: String,
    pub metadata_directive: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectRetentionRequest {
    pub bucket: BucketRef,
    pub object: ObjectRef,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectLegalHoldRequest {
    pub bucket: BucketRef,
    pub object: ObjectRef,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectExtractRequest {
    pub object: ObjectRef,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AppendObjectRejectedRequest {
    pub object: ObjectRef,
    #[serde(rename = "x-amz-write-offset-bytes")]
    pub write_offset_bytes: String,
    pub body: Vec<u8>,
}

pub struct PutObjectRequest {
    pub object: ObjectRef,
    pub body: crate::types::s3::core::BoxByteStream,
    pub content_type: Option<String>,
    pub content_md5: Option<String>,
}

impl std::fmt::Debug for PutObjectRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PutObjectRequest").field("object", &self.object).field("body", &"<stream>").finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteObjectRequest {
    pub object: ObjectRef,
    #[serde(rename = "versionId")]
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PostRestoreObjectRequest {
    pub object: ObjectRef,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketLocationRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketPolicyRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketLifecycleRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketEncryptionRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketObjectLockConfigRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketReplicationConfigRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketVersioningRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketNotificationRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListenBucketNotificationRequest {
    pub bucket: BucketRef,
    pub filter: EventFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResetBucketReplicationStatusRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketAclRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketAclRequest {
    pub bucket: BucketRef,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketCorsRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketCorsRequest {
    pub bucket: BucketRef,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketCorsRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketWebsiteRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketAccelerateRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketRequestPaymentRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketLoggingRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketTaggingRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketWebsiteRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketTaggingRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListMultipartUploadsRequest {
    pub bucket: BucketRef,
    pub query: ListQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectsV2MRequest {
    pub bucket: BucketRef,
    pub query: ListQuery,
    pub metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectsV2Request {
    pub bucket: BucketRef,
    pub query: ListQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectVersionsMRequest {
    pub bucket: BucketRef,
    pub query: ListQuery,
    pub metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectVersionsRequest {
    pub bucket: BucketRef,
    pub query: ListQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketPolicyStatusRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketLifecycleRequest {
    pub bucket: BucketRef,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketReplicationConfigRequest {
    pub bucket: BucketRef,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketEncryptionRequest {
    pub bucket: BucketRef,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketPolicyRequest {
    pub bucket: BucketRef,
    pub json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketObjectLockConfigRequest {
    pub bucket: BucketRef,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketTaggingRequest {
    pub bucket: BucketRef,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketVersioningRequest {
    pub bucket: BucketRef,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketNotificationRequest {
    pub bucket: BucketRef,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResetBucketReplicationStartRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketRequest {
    pub bucket: BucketRef,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HeadBucketRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PostPolicyRequest {
    pub bucket: BucketRef,
    pub form: PostPolicyForm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteMultipleObjectsRequest {
    pub bucket: BucketRef,
    pub payload: DeleteObjectsInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketPolicyRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketReplicationRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketLifecycleRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketEncryptionRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketReplicationMetricsV2Request {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketReplicationMetricsRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ValidateBucketReplicationCredsRequest {
    pub bucket: BucketRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectsV1Request {
    pub bucket: BucketRef,
    pub query: ListQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RootListenNotificationRequest {
    pub filter: EventFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListBucketsRequest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListBucketsDoubleSlashRequest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RejectedObjectTorrentRequest {
    pub object: ObjectRef,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RejectedObjectAclDeleteRequest {
    pub object: ObjectRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RejectedBucketApiRequest {
    pub bucket: BucketRef,
    pub api: String,
    pub method: String,
}

#[derive(Debug)]
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
