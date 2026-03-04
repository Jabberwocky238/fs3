use async_trait::async_trait;

use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{
    S3BucketConfigEngine, S3BucketEngine, S3MultipartEngine, S3ObjectEngine,
};
use crate::types::traits::s3_policyengine::{S3PolicyEngine, S3BucketPolicyEngine};
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;

use super::utils::*;

#[async_trait]
pub trait BucketS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    type Engine: S3BucketEngine + S3BucketConfigEngine + S3MultipartEngine + S3ObjectEngine + Send + Sync;
    type Policy: S3PolicyEngine;
    fn engine(&self) -> &Self::Engine;
    fn policy(&self) -> &Self::Policy;

    async fn get_bucket_location(&self, req: GetBucketLocationRequest) -> Result<GetBucketLocationResponse, E> {
        check_access(self.policy(), S3Action::GetBucketLocation, Some(&req.bucket.bucket), None).await?;
        let location = self.engine().get_bucket_location(&req.bucket.bucket).await?;
        Ok(GetBucketLocationResponse { location: Some(location), ..Default::default() })
    }
    async fn get_bucket_policy(&self, req: GetBucketPolicyRequest) -> Result<GetBucketPolicyResponse, E> {
        check_access(self.policy(), S3Action::GetBucketPolicy, Some(&req.bucket.bucket), None).await?;
        let p = self.policy().get_bucket_policy(&req.bucket.bucket).await
            .map_err(|e| S3EngineError::InvalidPolicy(e.to_string()))?;
        Ok(GetBucketPolicyResponse { config: p.unwrap_or_default(), ..Default::default() })
    }
    async fn get_bucket_lifecycle(&self, req: GetBucketLifecycleRequest) -> Result<GetBucketLifecycleResponse, E> {
        check_access(self.policy(), S3Action::GetBucketLifecycle, Some(&req.bucket.bucket), None).await?;
        let _p = self.engine().get_bucket_lifecycle(&req.bucket.bucket).await?;
        Ok(GetBucketLifecycleResponse { ..Default::default() })
    }
    async fn get_bucket_encryption(&self, req: GetBucketEncryptionRequest) -> Result<GetBucketEncryptionResponse, E> {
        check_access(self.policy(), S3Action::GetBucketEncryption, Some(&req.bucket.bucket), None).await?;
        let _p = self.engine().get_bucket_encryption(&req.bucket.bucket).await?;
        Ok(GetBucketEncryptionResponse { ..Default::default() })
    }
    async fn get_bucket_object_lock_config(&self, req: GetBucketObjectLockConfigRequest) -> Result<GetBucketObjectLockConfigResponse, E> {
        check_access(self.policy(), S3Action::GetBucketObjectLockConfiguration, Some(&req.bucket.bucket), None).await?;
        let _p = self.engine().get_bucket_object_lock_config(&req.bucket.bucket).await?;
        Ok(GetBucketObjectLockConfigResponse { ..Default::default() })
    }
    async fn get_bucket_replication_config(&self, req: GetBucketReplicationConfigRequest) -> Result<GetBucketReplicationConfigResponse, E> {
        check_access(self.policy(), S3Action::GetReplicationConfiguration, Some(&req.bucket.bucket), None).await?;
        let _p = self.engine().get_bucket_replication(&req.bucket.bucket).await?;
        Ok(GetBucketReplicationConfigResponse { ..Default::default() })
    }
    async fn get_bucket_versioning(&self, req: GetBucketVersioningRequest) -> Result<GetBucketVersioningResponse, E> {
        check_access(self.policy(), S3Action::GetBucketVersioning, Some(&req.bucket.bucket), None).await?;
        let _p = self.engine().get_bucket_versioning(&req.bucket.bucket).await?;
        Ok(GetBucketVersioningResponse { ..Default::default() })
    }
    async fn get_bucket_notification(&self, req: GetBucketNotificationRequest) -> Result<GetBucketNotificationResponse, E> {
        check_access(self.policy(), S3Action::GetBucketNotification, Some(&req.bucket.bucket), None).await?;
        let _p = self.engine().get_bucket_notification(&req.bucket.bucket).await?;
        Ok(GetBucketNotificationResponse { ..Default::default() })
    }
    async fn listen_bucket_notification(&self, _req: ListenBucketNotificationRequest) -> Result<ListenBucketNotificationResponse, E> { unsupported("ListenBucketNotification") }
    async fn reset_bucket_replication_status(&self, _req: ResetBucketReplicationStatusRequest) -> Result<ResetBucketReplicationStatusResponse, E> { unsupported("ResetBucketReplicationStatus") }
    async fn get_bucket_acl(&self, _req: GetBucketAclRequest) -> Result<GetBucketAclResponse, E> { Ok(Default::default()) }
    async fn put_bucket_acl(&self, _req: PutBucketAclRequest) -> Result<PutBucketAclResponse, E> { Ok(Default::default()) }
    async fn get_bucket_cors(&self, _req: GetBucketCorsRequest) -> Result<GetBucketCorsResponse, E> { Ok(Default::default()) }
    async fn put_bucket_cors(&self, _req: PutBucketCorsRequest) -> Result<PutBucketCorsResponse, E> { Ok(Default::default()) }
    async fn delete_bucket_cors(&self, _req: DeleteBucketCorsRequest) -> Result<DeleteBucketCorsResponse, E> { Ok(Default::default()) }
    async fn get_bucket_website(&self, _req: GetBucketWebsiteRequest) -> Result<GetBucketWebsiteResponse, E> { Ok(Default::default()) }
    async fn get_bucket_accelerate(&self, _req: GetBucketAccelerateRequest) -> Result<GetBucketAccelerateResponse, E> { Ok(Default::default()) }
    async fn get_bucket_request_payment(&self, _req: GetBucketRequestPaymentRequest) -> Result<GetBucketRequestPaymentResponse, E> { Ok(Default::default()) }
    async fn get_bucket_logging(&self, _req: GetBucketLoggingRequest) -> Result<GetBucketLoggingResponse, E> { Ok(Default::default()) }
    async fn get_bucket_tagging(&self, req: GetBucketTaggingRequest) -> Result<GetBucketTaggingResponse, E> {
        check_access(self.policy(), S3Action::GetBucketTagging, Some(&req.bucket.bucket), None).await?;
        let _p = self.engine().get_bucket_tagging(&req.bucket.bucket).await?;
        Ok(GetBucketTaggingResponse { tags: Default::default(), ..Default::default() })
    }
    async fn delete_bucket_website(&self, _req: DeleteBucketWebsiteRequest) -> Result<DeleteBucketWebsiteResponse, E> { Ok(Default::default()) }
    async fn delete_bucket_tagging(&self, req: DeleteBucketTaggingRequest) -> Result<DeleteBucketTaggingResponse, E> {
        check_access(self.policy(), S3Action::PutBucketTagging, Some(&req.bucket.bucket), None).await?;
        self.engine().delete_bucket_tagging(&req.bucket.bucket).await?;
        Ok(Default::default())
    }
    async fn list_multipart_uploads(&self, req: ListMultipartUploadsRequest) -> Result<ListMultipartUploadsResponse, E> {
        check_access(self.policy(), S3Action::ListBucketMultipartUploads, Some(&req.bucket.bucket), None).await?;
        let uploads = self.engine().list_multipart_uploads(&req.bucket.bucket, to_list_opt(&req.query, false)).await?;
        Ok(ListMultipartUploadsResponse {
            uploads: uploads.into_iter().map(|u| MultipartUploadInfo {
                key: u.key,
                upload_id: u.upload_id,
                initiated: Some(u.initiated_at.to_rfc3339()),
            }).collect(),
            ..Default::default()
        })
    }
    async fn list_objects_v2m(&self, req: ListObjectsV2MRequest) -> Result<ListObjectsV2MResponse, E> {
        check_access(self.policy(), S3Action::ListBucket, Some(&req.bucket.bucket), None).await?;
        let p = self.engine().list_objects_v2(&req.bucket.bucket, to_list_opt(&req.query, req.metadata)).await?;
        Ok(ListObjectsV2MResponse {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }
    async fn list_objects_v2(&self, req: ListObjectsV2Request) -> Result<ListObjectsV2Response, E> {
        check_access(self.policy(), S3Action::ListBucket, Some(&req.bucket.bucket), None).await?;
        let p = self.engine().list_objects_v2(&req.bucket.bucket, to_list_opt(&req.query, false)).await?;
        Ok(ListObjectsV2Response { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
    async fn list_object_versions_m(&self, req: ListObjectVersionsMRequest) -> Result<ListObjectVersionsMResponse, E> {
        check_access(self.policy(), S3Action::ListBucketVersions, Some(&req.bucket.bucket), None).await?;
        let p = self.engine().list_object_versions(&req.bucket.bucket, to_list_opt(&req.query, req.metadata)).await?;
        Ok(ListObjectVersionsMResponse { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
    async fn list_object_versions(&self, req: ListObjectVersionsRequest) -> Result<ListObjectVersionsResponse, E> {
        let p = self.engine().list_object_versions(&req.bucket.bucket, to_list_opt(&req.query, false)).await?;
        Ok(ListObjectVersionsResponse { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
    async fn get_bucket_policy_status(&self, req: GetBucketPolicyStatusRequest) -> Result<GetBucketPolicyStatusResponse, E> {
        check_access(self.policy(), S3Action::GetBucketPolicyStatus, Some(&req.bucket.bucket), None).await?;
        let p = self.policy().get_bucket_policy(&req.bucket.bucket).await
            .map_err(|e| S3EngineError::InvalidPolicy(e.to_string()))?;
        let is_public = p.as_ref()
            .map(|d| d.to_ascii_lowercase().contains("\"effect\":\"allow\""))
            .unwrap_or(false);
        Ok(GetBucketPolicyStatusResponse { is_public: Some(is_public), ..Default::default() })
    }
    async fn put_bucket_lifecycle(&self, req: PutBucketLifecycleRequest) -> Result<PutBucketLifecycleResponse, E> {
        check_access(self.policy(), S3Action::PutBucketLifecycle, Some(&req.bucket.bucket), None).await?;
        self.engine().put_bucket_lifecycle(&req.bucket.bucket, req.xml).await?;
        Ok(Default::default())
    }
    async fn put_bucket_replication_config(&self, req: PutBucketReplicationConfigRequest) -> Result<PutBucketReplicationConfigResponse, E> {
        check_access(self.policy(), S3Action::PutReplicationConfiguration, Some(&req.bucket.bucket), None).await?;
        self.engine().put_bucket_replication(&req.bucket.bucket, req.xml).await?;
        Ok(Default::default())
    }
    async fn put_bucket_encryption(&self, req: PutBucketEncryptionRequest) -> Result<PutBucketEncryptionResponse, E> {
        check_access(self.policy(), S3Action::PutBucketEncryption, Some(&req.bucket.bucket), None).await?;
        self.engine().put_bucket_encryption(&req.bucket.bucket, req.xml).await?;
        Ok(Default::default())
    }
    async fn put_bucket_policy(&self, req: PutBucketPolicyRequest) -> Result<PutBucketPolicyResponse, E> {
        check_access(self.policy(), S3Action::PutBucketPolicy, Some(&req.bucket.bucket), None).await?;
        self.policy().put_bucket_policy(&req.bucket.bucket, &req.json).await
            .map_err(|e| S3EngineError::InvalidPolicy(e.to_string()))?;
        Ok(Default::default())
    }
    async fn put_bucket_object_lock_config(&self, req: PutBucketObjectLockConfigRequest) -> Result<PutBucketObjectLockConfigResponse, E> {
        check_access(self.policy(), S3Action::PutBucketObjectLockConfiguration, Some(&req.bucket.bucket), None).await?;
        self.engine().put_bucket_object_lock_config(&req.bucket.bucket, req.xml).await?;
        Ok(Default::default())
    }
    async fn put_bucket_tagging(&self, req: PutBucketTaggingRequest) -> Result<PutBucketTaggingResponse, E> {
        check_access(self.policy(), S3Action::PutBucketTagging, Some(&req.bucket.bucket), None).await?;
        self.engine().put_bucket_tagging(&req.bucket.bucket, req.xml).await?;
        Ok(Default::default())
    }
    async fn put_bucket_versioning(&self, req: PutBucketVersioningRequest) -> Result<PutBucketVersioningResponse, E> {
        check_access(self.policy(), S3Action::PutBucketVersioning, Some(&req.bucket.bucket), None).await?;
        self.engine().put_bucket_versioning(&req.bucket.bucket, req.xml).await?;
        Ok(Default::default())
    }
    async fn put_bucket_notification(&self, req: PutBucketNotificationRequest) -> Result<PutBucketNotificationResponse, E> {
        check_access(self.policy(), S3Action::PutBucketNotification, Some(&req.bucket.bucket), None).await?;
        self.engine().put_bucket_notification(&req.bucket.bucket, req.xml).await?;
        Ok(Default::default())
    }
    async fn reset_bucket_replication_start(&self, _req: ResetBucketReplicationStartRequest) -> Result<ResetBucketReplicationStartResponse, E> { unsupported("ResetBucketReplicationStart") }
    async fn put_bucket(&self, req: PutBucketRequest) -> Result<PutBucketResponse, E> {
        check_access(self.policy(), S3Action::CreateBucket, Some(&req.bucket.bucket), None).await?;
        let _ = self
            .engine()
            .make_bucket(&req.bucket.bucket, req.region.as_deref(), bucket_features_for_create())
            .await
            ?;
        Ok(Default::default())
    }
    async fn head_bucket(&self, req: HeadBucketRequest) -> Result<HeadBucketResponse, E> {
        check_access(self.policy(), S3Action::HeadBucket, Some(&req.bucket.bucket), None).await?;
        let _ = self.engine().head_bucket(&req.bucket.bucket).await?;
        Ok(Default::default())
    }
    async fn post_policy(&self, _req: PostPolicyRequest) -> Result<PostPolicyResponse, E> { unsupported("PostPolicy") }
    async fn delete_multiple_objects(&self, req: DeleteMultipleObjectsRequest) -> Result<DeleteMultipleObjectsResponse, E> {
        check_access(self.policy(), S3Action::DeleteObject, Some(&req.bucket.bucket), None).await?;
        let keys = parse_delete_keys(&req.payload.xml);
        if keys.is_empty() {
            return Err(S3HandlerBridgeError::InvalidRequest("DeleteMultipleObjects payload has no <Key>".to_string()).into());
        }
        let r = self
            .engine()
            .delete_objects(&req.bucket.bucket, keys, to_delete_opt(None))
            .await
            ?;
        Ok(DeleteMultipleObjectsResponse {
            deleted: r.deleted.into_iter().filter_map(|d| d.version_id).collect(),
            errors: r
                .errors
                .into_iter()
                .map(|e| ErrorBody {
                    code: e.code,
                    message: e.message,
                    resource: e.key,
                })
                .collect(),
            ..Default::default()
        })
    }
    async fn delete_bucket_policy(&self, req: DeleteBucketPolicyRequest) -> Result<DeleteBucketPolicyResponse, E> {
        check_access(self.policy(), S3Action::DeleteBucketPolicy, Some(&req.bucket.bucket), None).await?;
        self.policy().delete_bucket_policy(&req.bucket.bucket).await
            .map_err(|e| S3EngineError::InvalidPolicy(e.to_string()))?;
        Ok(Default::default())
    }
    async fn delete_bucket_replication(&self, req: DeleteBucketReplicationRequest) -> Result<DeleteBucketReplicationResponse, E> {
        check_access(self.policy(), S3Action::PutReplicationConfiguration, Some(&req.bucket.bucket), None).await?;
        self.engine().delete_bucket_replication(&req.bucket.bucket).await?;
        Ok(Default::default())
    }
    async fn delete_bucket_lifecycle(&self, req: DeleteBucketLifecycleRequest) -> Result<DeleteBucketLifecycleResponse, E> {
        check_access(self.policy(), S3Action::PutBucketLifecycle, Some(&req.bucket.bucket), None).await?;
        self.engine().delete_bucket_lifecycle(&req.bucket.bucket).await?;
        Ok(Default::default())
    }
    async fn delete_bucket_encryption(&self, req: DeleteBucketEncryptionRequest) -> Result<DeleteBucketEncryptionResponse, E> {
        check_access(self.policy(), S3Action::PutBucketEncryption, Some(&req.bucket.bucket), None).await?;
        self.engine().delete_bucket_encryption(&req.bucket.bucket).await?;
        Ok(Default::default())
    }
    async fn delete_bucket(&self, req: DeleteBucketRequest) -> Result<DeleteBucketResponse, E> {
        check_access(self.policy(), S3Action::DeleteBucket, Some(&req.bucket.bucket), None).await?;
        self.engine().delete_bucket(&req.bucket.bucket, false).await?;
        Ok(Default::default())
    }
    async fn get_bucket_replication_metrics_v2(&self, req: GetBucketReplicationMetricsV2Request) -> Result<GetBucketReplicationMetricsV2Response, E> {
        let r = self.engine().get_bucket_replication_metrics(&req.bucket.bucket).await?;
        Ok(GetBucketReplicationMetricsV2Response { ..Default::default() })
    }
    async fn get_bucket_replication_metrics(&self, req: GetBucketReplicationMetricsRequest) -> Result<GetBucketReplicationMetricsResponse, E> {
        let r = self.engine().get_bucket_replication_metrics(&req.bucket.bucket).await?;
        Ok(GetBucketReplicationMetricsResponse { ..Default::default() })
    }
    async fn validate_bucket_replication_creds(&self, req: ValidateBucketReplicationCredsRequest) -> Result<ValidateBucketReplicationCredsResponse, E> {
        let v = self.engine().validate_bucket_replication_creds(&req.bucket.bucket).await?;
        Ok(ValidateBucketReplicationCredsResponse { valid: v.valid, ..Default::default() })
    }
    async fn list_objects_v1(&self, req: ListObjectsV1Request) -> Result<ListObjectsV1Response, E> {
        check_access(self.policy(), S3Action::ListBucket, Some(&req.bucket.bucket), None).await?;
        let p = self.engine().list_objects_v1(&req.bucket.bucket, to_list_opt(&req.query, false)).await?;
        Ok(ListObjectsV1Response { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
}
