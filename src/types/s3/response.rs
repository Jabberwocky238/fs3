use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type HeaderMap = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResponseMeta {
    pub status_code: u16,
    pub request_id: Option<String>,
    pub host_id: Option<String>,
    pub etag: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ObjectInfo {
    pub bucket: String,
    pub key: String,
    pub size: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub storage_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BucketInfo {
    pub name: String,
    pub creation_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub resource: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MultipartPartInfo {
    pub part_number: u32,
    pub etag: Option<String>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MultipartUploadInfo {
    pub key: String,
    pub upload_id: String,
    pub initiated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NotificationEventRecord {
    pub event_name: String,
    pub event_time: String,
    pub bucket: String,
    pub object: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BaseOkResponse {
    pub meta: ResponseMeta,
    pub headers: HeaderMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HeadObjectResponse {
    pub meta: ResponseMeta,
    pub headers: HeaderMap,
    pub object: Option<ObjectInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectAttributesResponse {
    pub meta: ResponseMeta,
    pub headers: HeaderMap,
    pub object: Option<ObjectInfo>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CopyObjectPartResponse {
    pub meta: ResponseMeta,
    pub part: Option<MultipartPartInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectPartResponse {
    pub meta: ResponseMeta,
    pub part: Option<MultipartPartInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectPartsResponse {
    pub meta: ResponseMeta,
    #[serde(rename = "uploadId")]
    pub upload_id: Option<String>,
    pub parts: Vec<MultipartPartInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CompleteMultipartUploadResponse {
    pub meta: ResponseMeta,
    pub object: Option<ObjectInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NewMultipartUploadResponse {
    pub meta: ResponseMeta,
    #[serde(rename = "uploadId")]
    pub upload_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AbortMultipartUploadResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectAclResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectAclResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectTaggingResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectTaggingResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteObjectTaggingResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SelectObjectContentResponse {
    pub meta: ResponseMeta,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectRetentionResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectLegalHoldResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetObjectLambdaResponse {
    pub meta: ResponseMeta,
    pub body: Vec<u8>,
}

pub struct GetObjectResponse {
    pub meta: ResponseMeta,
    pub size: Option<u64>,
    pub body: crate::types::s3::core::BoxByteStream,
}

impl std::fmt::Debug for GetObjectResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetObjectResponse").field("meta", &self.meta).field("size", &self.size).field("body", &"<stream>").finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CopyObjectResponse {
    pub meta: ResponseMeta,
    pub object: Option<ObjectInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectRetentionResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectLegalHoldResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectExtractResponse {
    pub meta: ResponseMeta,
    pub extracted_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AppendObjectRejectedResponse {
    pub meta: ResponseMeta,
    pub error: ErrorBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutObjectResponse {
    pub meta: ResponseMeta,
    pub object: Option<ObjectInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteObjectResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PostRestoreObjectResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketLocationResponse {
    pub meta: ResponseMeta,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketPolicyResponse {
    pub meta: ResponseMeta,
    pub json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketLifecycleResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketEncryptionResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketObjectLockConfigResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketReplicationConfigResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketVersioningResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketNotificationResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListenBucketNotificationResponse {
    pub meta: ResponseMeta,
    pub records: Vec<NotificationEventRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResetBucketReplicationStatusResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketAclResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketAclResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketCorsResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketCorsResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketCorsResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketWebsiteResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketAccelerateResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketRequestPaymentResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketLoggingResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketTaggingResponse {
    pub meta: ResponseMeta,
    pub xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketWebsiteResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketTaggingResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListMultipartUploadsResponse {
    pub meta: ResponseMeta,
    pub uploads: Vec<MultipartUploadInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectsV2MResponse {
    pub meta: ResponseMeta,
    pub objects: Vec<ObjectInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectsV2Response {
    pub meta: ResponseMeta,
    pub objects: Vec<ObjectInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectVersionsMResponse {
    pub meta: ResponseMeta,
    pub objects: Vec<ObjectInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectVersionsResponse {
    pub meta: ResponseMeta,
    pub objects: Vec<ObjectInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketPolicyStatusResponse {
    pub meta: ResponseMeta,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketLifecycleResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketReplicationConfigResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketEncryptionResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketPolicyResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketObjectLockConfigResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketTaggingResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketVersioningResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketNotificationResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResetBucketReplicationStartResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PutBucketResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HeadBucketResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PostPolicyResponse {
    pub meta: ResponseMeta,
    pub key: Option<String>,
    pub bucket: Option<String>,
    pub location: Option<String>,
    pub etag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteMultipleObjectsResponse {
    pub meta: ResponseMeta,
    pub deleted: Vec<String>,
    pub errors: Vec<ErrorBody>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketPolicyResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketReplicationResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketLifecycleResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketEncryptionResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteBucketResponse {
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketReplicationMetricsV2Response {
    pub meta: ResponseMeta,
    pub json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetBucketReplicationMetricsResponse {
    pub meta: ResponseMeta,
    pub json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ValidateBucketReplicationCredsResponse {
    pub meta: ResponseMeta,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListObjectsV1Response {
    pub meta: ResponseMeta,
    pub objects: Vec<ObjectInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RootListenNotificationResponse {
    pub meta: ResponseMeta,
    pub records: Vec<NotificationEventRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListBucketsResponse {
    pub meta: ResponseMeta,
    pub buckets: Vec<BucketInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListBucketsDoubleSlashResponse {
    pub meta: ResponseMeta,
    pub buckets: Vec<BucketInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RejectedApiResponse {
    pub meta: ResponseMeta,
    pub error: ErrorBody,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum S3Response {
    HeadObject(HeadObjectResponse),
    GetObjectAttributes(GetObjectAttributesResponse),
    CopyObjectPart(CopyObjectPartResponse),
    PutObjectPart(PutObjectPartResponse),
    ListObjectParts(ListObjectPartsResponse),
    CompleteMultipartUpload(CompleteMultipartUploadResponse),
    NewMultipartUpload(NewMultipartUploadResponse),
    AbortMultipartUpload(AbortMultipartUploadResponse),
    GetObjectAcl(GetObjectAclResponse),
    PutObjectAcl(PutObjectAclResponse),
    GetObjectTagging(GetObjectTaggingResponse),
    PutObjectTagging(PutObjectTaggingResponse),
    DeleteObjectTagging(DeleteObjectTaggingResponse),
    SelectObjectContent(SelectObjectContentResponse),
    GetObjectRetention(GetObjectRetentionResponse),
    GetObjectLegalHold(GetObjectLegalHoldResponse),
    GetObjectLambda(GetObjectLambdaResponse),
    GetObject(GetObjectResponse),
    CopyObject(CopyObjectResponse),
    PutObjectRetention(PutObjectRetentionResponse),
    PutObjectLegalHold(PutObjectLegalHoldResponse),
    PutObjectExtract(PutObjectExtractResponse),
    AppendObjectRejected(AppendObjectRejectedResponse),
    PutObject(PutObjectResponse),
    DeleteObject(DeleteObjectResponse),
    PostRestoreObject(PostRestoreObjectResponse),
    GetBucketLocation(GetBucketLocationResponse),
    GetBucketPolicy(GetBucketPolicyResponse),
    GetBucketLifecycle(GetBucketLifecycleResponse),
    GetBucketEncryption(GetBucketEncryptionResponse),
    GetBucketObjectLockConfig(GetBucketObjectLockConfigResponse),
    GetBucketReplicationConfig(GetBucketReplicationConfigResponse),
    GetBucketVersioning(GetBucketVersioningResponse),
    GetBucketNotification(GetBucketNotificationResponse),
    ListenBucketNotification(ListenBucketNotificationResponse),
    ResetBucketReplicationStatus(ResetBucketReplicationStatusResponse),
    GetBucketAcl(GetBucketAclResponse),
    PutBucketAcl(PutBucketAclResponse),
    GetBucketCors(GetBucketCorsResponse),
    PutBucketCors(PutBucketCorsResponse),
    DeleteBucketCors(DeleteBucketCorsResponse),
    GetBucketWebsite(GetBucketWebsiteResponse),
    GetBucketAccelerate(GetBucketAccelerateResponse),
    GetBucketRequestPayment(GetBucketRequestPaymentResponse),
    GetBucketLogging(GetBucketLoggingResponse),
    GetBucketTagging(GetBucketTaggingResponse),
    DeleteBucketWebsite(DeleteBucketWebsiteResponse),
    DeleteBucketTagging(DeleteBucketTaggingResponse),
    ListMultipartUploads(ListMultipartUploadsResponse),
    ListObjectsV2M(ListObjectsV2MResponse),
    ListObjectsV2(ListObjectsV2Response),
    ListObjectVersionsM(ListObjectVersionsMResponse),
    ListObjectVersions(ListObjectVersionsResponse),
    GetBucketPolicyStatus(GetBucketPolicyStatusResponse),
    PutBucketLifecycle(PutBucketLifecycleResponse),
    PutBucketReplicationConfig(PutBucketReplicationConfigResponse),
    PutBucketEncryption(PutBucketEncryptionResponse),
    PutBucketPolicy(PutBucketPolicyResponse),
    PutBucketObjectLockConfig(PutBucketObjectLockConfigResponse),
    PutBucketTagging(PutBucketTaggingResponse),
    PutBucketVersioning(PutBucketVersioningResponse),
    PutBucketNotification(PutBucketNotificationResponse),
    ResetBucketReplicationStart(ResetBucketReplicationStartResponse),
    PutBucket(PutBucketResponse),
    HeadBucket(HeadBucketResponse),
    PostPolicy(PostPolicyResponse),
    DeleteMultipleObjects(DeleteMultipleObjectsResponse),
    DeleteBucketPolicy(DeleteBucketPolicyResponse),
    DeleteBucketReplication(DeleteBucketReplicationResponse),
    DeleteBucketLifecycle(DeleteBucketLifecycleResponse),
    DeleteBucketEncryption(DeleteBucketEncryptionResponse),
    DeleteBucket(DeleteBucketResponse),
    GetBucketReplicationMetricsV2(GetBucketReplicationMetricsV2Response),
    GetBucketReplicationMetrics(GetBucketReplicationMetricsResponse),
    ValidateBucketReplicationCreds(ValidateBucketReplicationCredsResponse),
    ListObjectsV1(ListObjectsV1Response),
    RootListenNotification(RootListenNotificationResponse),
    ListBuckets(ListBucketsResponse),
    ListBucketsDoubleSlash(ListBucketsDoubleSlashResponse),
    RejectedApi(RejectedApiResponse),
}

