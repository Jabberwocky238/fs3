use async_trait::async_trait;

use crate::types::s3::*;

#[async_trait]
pub trait ObjectS3Handler {
    type Error: Send + Sync + 'static;

    async fn head_object(&self, req: HeadObjectRequest) -> Result<HeadObjectResponse, Self::Error>;
    async fn get_object_attributes(
        &self,
        req: GetObjectAttributesRequest,
    ) -> Result<GetObjectAttributesResponse, Self::Error>;
    async fn copy_object_part(
        &self,
        req: CopyObjectPartRequest,
    ) -> Result<CopyObjectPartResponse, Self::Error>;
    async fn put_object_part(
        &self,
        req: PutObjectPartRequest,
    ) -> Result<PutObjectPartResponse, Self::Error>;
    async fn list_object_parts(
        &self,
        req: ListObjectPartsRequest,
    ) -> Result<ListObjectPartsResponse, Self::Error>;
    async fn complete_multipart_upload(
        &self,
        req: CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResponse, Self::Error>;
    async fn new_multipart_upload(
        &self,
        req: NewMultipartUploadRequest,
    ) -> Result<NewMultipartUploadResponse, Self::Error>;
    async fn abort_multipart_upload(
        &self,
        req: AbortMultipartUploadRequest,
    ) -> Result<AbortMultipartUploadResponse, Self::Error>;
    async fn get_object_acl(
        &self,
        req: GetObjectAclRequest,
    ) -> Result<GetObjectAclResponse, Self::Error>;
    async fn put_object_acl(
        &self,
        req: PutObjectAclRequest,
    ) -> Result<PutObjectAclResponse, Self::Error>;
    async fn get_object_tagging(
        &self,
        req: GetObjectTaggingRequest,
    ) -> Result<GetObjectTaggingResponse, Self::Error>;
    async fn put_object_tagging(
        &self,
        req: PutObjectTaggingRequest,
    ) -> Result<PutObjectTaggingResponse, Self::Error>;
    async fn delete_object_tagging(
        &self,
        req: DeleteObjectTaggingRequest,
    ) -> Result<DeleteObjectTaggingResponse, Self::Error>;
    async fn select_object_content(
        &self,
        req: SelectObjectContentRequest,
    ) -> Result<SelectObjectContentResponse, Self::Error>;
    async fn get_object_retention(
        &self,
        req: GetObjectRetentionRequest,
    ) -> Result<GetObjectRetentionResponse, Self::Error>;
    async fn get_object_legal_hold(
        &self,
        req: GetObjectLegalHoldRequest,
    ) -> Result<GetObjectLegalHoldResponse, Self::Error>;
    async fn get_object_lambda(
        &self,
        req: GetObjectLambdaRequest,
    ) -> Result<GetObjectLambdaResponse, Self::Error>;
    async fn get_object(&self, req: GetObjectRequest) -> Result<GetObjectResponse, Self::Error>;
    async fn copy_object(&self, req: CopyObjectRequest) -> Result<CopyObjectResponse, Self::Error>;
    async fn put_object_retention(
        &self,
        req: PutObjectRetentionRequest,
    ) -> Result<PutObjectRetentionResponse, Self::Error>;
    async fn put_object_legal_hold(
        &self,
        req: PutObjectLegalHoldRequest,
    ) -> Result<PutObjectLegalHoldResponse, Self::Error>;
    async fn put_object_extract(
        &self,
        req: PutObjectExtractRequest,
    ) -> Result<PutObjectExtractResponse, Self::Error>;
    async fn append_object_rejected(
        &self,
        req: AppendObjectRejectedRequest,
    ) -> Result<AppendObjectRejectedResponse, Self::Error>;
    async fn put_object(&self, req: PutObjectRequest) -> Result<PutObjectResponse, Self::Error>;
    async fn delete_object(
        &self,
        req: DeleteObjectRequest,
    ) -> Result<DeleteObjectResponse, Self::Error>;
    async fn post_restore_object(
        &self,
        req: PostRestoreObjectRequest,
    ) -> Result<PostRestoreObjectResponse, Self::Error>;
}

#[async_trait]
pub trait BucketS3Handler {
    type Error: Send + Sync + 'static;

    async fn get_bucket_location(
        &self,
        req: GetBucketLocationRequest,
    ) -> Result<GetBucketLocationResponse, Self::Error>;
    async fn get_bucket_policy(
        &self,
        req: GetBucketPolicyRequest,
    ) -> Result<GetBucketPolicyResponse, Self::Error>;
    async fn get_bucket_lifecycle(
        &self,
        req: GetBucketLifecycleRequest,
    ) -> Result<GetBucketLifecycleResponse, Self::Error>;
    async fn get_bucket_encryption(
        &self,
        req: GetBucketEncryptionRequest,
    ) -> Result<GetBucketEncryptionResponse, Self::Error>;
    async fn get_bucket_object_lock_config(
        &self,
        req: GetBucketObjectLockConfigRequest,
    ) -> Result<GetBucketObjectLockConfigResponse, Self::Error>;
    async fn get_bucket_replication_config(
        &self,
        req: GetBucketReplicationConfigRequest,
    ) -> Result<GetBucketReplicationConfigResponse, Self::Error>;
    async fn get_bucket_versioning(
        &self,
        req: GetBucketVersioningRequest,
    ) -> Result<GetBucketVersioningResponse, Self::Error>;
    async fn get_bucket_notification(
        &self,
        req: GetBucketNotificationRequest,
    ) -> Result<GetBucketNotificationResponse, Self::Error>;
    async fn listen_bucket_notification(
        &self,
        req: ListenBucketNotificationRequest,
    ) -> Result<ListenBucketNotificationResponse, Self::Error>;
    async fn reset_bucket_replication_status(
        &self,
        req: ResetBucketReplicationStatusRequest,
    ) -> Result<ResetBucketReplicationStatusResponse, Self::Error>;
    async fn get_bucket_acl(
        &self,
        req: GetBucketAclRequest,
    ) -> Result<GetBucketAclResponse, Self::Error>;
    async fn put_bucket_acl(
        &self,
        req: PutBucketAclRequest,
    ) -> Result<PutBucketAclResponse, Self::Error>;
    async fn get_bucket_cors(
        &self,
        req: GetBucketCorsRequest,
    ) -> Result<GetBucketCorsResponse, Self::Error>;
    async fn put_bucket_cors(
        &self,
        req: PutBucketCorsRequest,
    ) -> Result<PutBucketCorsResponse, Self::Error>;
    async fn delete_bucket_cors(
        &self,
        req: DeleteBucketCorsRequest,
    ) -> Result<DeleteBucketCorsResponse, Self::Error>;
    async fn get_bucket_website(
        &self,
        req: GetBucketWebsiteRequest,
    ) -> Result<GetBucketWebsiteResponse, Self::Error>;
    async fn get_bucket_accelerate(
        &self,
        req: GetBucketAccelerateRequest,
    ) -> Result<GetBucketAccelerateResponse, Self::Error>;
    async fn get_bucket_request_payment(
        &self,
        req: GetBucketRequestPaymentRequest,
    ) -> Result<GetBucketRequestPaymentResponse, Self::Error>;
    async fn get_bucket_logging(
        &self,
        req: GetBucketLoggingRequest,
    ) -> Result<GetBucketLoggingResponse, Self::Error>;
    async fn get_bucket_tagging(
        &self,
        req: GetBucketTaggingRequest,
    ) -> Result<GetBucketTaggingResponse, Self::Error>;
    async fn delete_bucket_website(
        &self,
        req: DeleteBucketWebsiteRequest,
    ) -> Result<DeleteBucketWebsiteResponse, Self::Error>;
    async fn delete_bucket_tagging(
        &self,
        req: DeleteBucketTaggingRequest,
    ) -> Result<DeleteBucketTaggingResponse, Self::Error>;
    async fn list_multipart_uploads(
        &self,
        req: ListMultipartUploadsRequest,
    ) -> Result<ListMultipartUploadsResponse, Self::Error>;
    async fn list_objects_v2m(
        &self,
        req: ListObjectsV2MRequest,
    ) -> Result<ListObjectsV2MResponse, Self::Error>;
    async fn list_objects_v2(
        &self,
        req: ListObjectsV2Request,
    ) -> Result<ListObjectsV2Response, Self::Error>;
    async fn list_object_versions_m(
        &self,
        req: ListObjectVersionsMRequest,
    ) -> Result<ListObjectVersionsMResponse, Self::Error>;
    async fn list_object_versions(
        &self,
        req: ListObjectVersionsRequest,
    ) -> Result<ListObjectVersionsResponse, Self::Error>;
    async fn get_bucket_policy_status(
        &self,
        req: GetBucketPolicyStatusRequest,
    ) -> Result<GetBucketPolicyStatusResponse, Self::Error>;
    async fn put_bucket_lifecycle(
        &self,
        req: PutBucketLifecycleRequest,
    ) -> Result<PutBucketLifecycleResponse, Self::Error>;
    async fn put_bucket_replication_config(
        &self,
        req: PutBucketReplicationConfigRequest,
    ) -> Result<PutBucketReplicationConfigResponse, Self::Error>;
    async fn put_bucket_encryption(
        &self,
        req: PutBucketEncryptionRequest,
    ) -> Result<PutBucketEncryptionResponse, Self::Error>;
    async fn put_bucket_policy(
        &self,
        req: PutBucketPolicyRequest,
    ) -> Result<PutBucketPolicyResponse, Self::Error>;
    async fn put_bucket_object_lock_config(
        &self,
        req: PutBucketObjectLockConfigRequest,
    ) -> Result<PutBucketObjectLockConfigResponse, Self::Error>;
    async fn put_bucket_tagging(
        &self,
        req: PutBucketTaggingRequest,
    ) -> Result<PutBucketTaggingResponse, Self::Error>;
    async fn put_bucket_versioning(
        &self,
        req: PutBucketVersioningRequest,
    ) -> Result<PutBucketVersioningResponse, Self::Error>;
    async fn put_bucket_notification(
        &self,
        req: PutBucketNotificationRequest,
    ) -> Result<PutBucketNotificationResponse, Self::Error>;
    async fn reset_bucket_replication_start(
        &self,
        req: ResetBucketReplicationStartRequest,
    ) -> Result<ResetBucketReplicationStartResponse, Self::Error>;
    async fn put_bucket(&self, req: PutBucketRequest) -> Result<PutBucketResponse, Self::Error>;
    async fn head_bucket(&self, req: HeadBucketRequest) -> Result<HeadBucketResponse, Self::Error>;
    async fn post_policy(&self, req: PostPolicyRequest) -> Result<PostPolicyResponse, Self::Error>;
    async fn delete_multiple_objects(
        &self,
        req: DeleteMultipleObjectsRequest,
    ) -> Result<DeleteMultipleObjectsResponse, Self::Error>;
    async fn delete_bucket_policy(
        &self,
        req: DeleteBucketPolicyRequest,
    ) -> Result<DeleteBucketPolicyResponse, Self::Error>;
    async fn delete_bucket_replication(
        &self,
        req: DeleteBucketReplicationRequest,
    ) -> Result<DeleteBucketReplicationResponse, Self::Error>;
    async fn delete_bucket_lifecycle(
        &self,
        req: DeleteBucketLifecycleRequest,
    ) -> Result<DeleteBucketLifecycleResponse, Self::Error>;
    async fn delete_bucket_encryption(
        &self,
        req: DeleteBucketEncryptionRequest,
    ) -> Result<DeleteBucketEncryptionResponse, Self::Error>;
    async fn delete_bucket(
        &self,
        req: DeleteBucketRequest,
    ) -> Result<DeleteBucketResponse, Self::Error>;
    async fn get_bucket_replication_metrics_v2(
        &self,
        req: GetBucketReplicationMetricsV2Request,
    ) -> Result<GetBucketReplicationMetricsV2Response, Self::Error>;
    async fn get_bucket_replication_metrics(
        &self,
        req: GetBucketReplicationMetricsRequest,
    ) -> Result<GetBucketReplicationMetricsResponse, Self::Error>;
    async fn validate_bucket_replication_creds(
        &self,
        req: ValidateBucketReplicationCredsRequest,
    ) -> Result<ValidateBucketReplicationCredsResponse, Self::Error>;
    async fn list_objects_v1(
        &self,
        req: ListObjectsV1Request,
    ) -> Result<ListObjectsV1Response, Self::Error>;
}

#[async_trait]
pub trait RootS3Handler {
    type Error: Send + Sync + 'static;

    async fn root_listen_notification(
        &self,
        req: RootListenNotificationRequest,
    ) -> Result<RootListenNotificationResponse, Self::Error>;
    async fn list_buckets(
        &self,
        req: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, Self::Error>;
    async fn list_buckets_double_slash(
        &self,
        req: ListBucketsDoubleSlashRequest,
    ) -> Result<ListBucketsDoubleSlashResponse, Self::Error>;
}

#[async_trait]
pub trait RejectedS3Handler {
    type Error: Send + Sync + 'static;

    async fn rejected_object_torrent(
        &self,
        req: RejectedObjectTorrentRequest,
    ) -> Result<RejectedApiResponse, Self::Error>;
    async fn rejected_object_acl_delete(
        &self,
        req: RejectedObjectAclDeleteRequest,
    ) -> Result<RejectedApiResponse, Self::Error>;
    async fn rejected_bucket_api(
        &self,
        req: RejectedBucketApiRequest,
    ) -> Result<RejectedApiResponse, Self::Error>;
}
