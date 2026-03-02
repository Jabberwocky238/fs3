use async_trait::async_trait;

use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{
    S3BucketConfigEngine, S3BucketEngine, S3MultipartEngine, S3ObjectEngine,
};

use super::utils::*;

#[async_trait]
pub trait BucketS3Handler
where
    <Self::Engine as S3BucketEngine>::Error: Into<Self::Error>,
    <Self::Engine as S3BucketConfigEngine>::Error: Into<Self::Error>,
    <Self::Engine as S3MultipartEngine>::Error: Into<Self::Error>,
    <Self::Engine as S3ObjectEngine>::Error: Into<Self::Error>,
    Self::Error: From<S3HandlerBridgeError>,
{
    type Engine: S3BucketEngine + S3BucketConfigEngine + S3MultipartEngine + S3ObjectEngine;
    type Error: Send + Sync + 'static;
    fn engine(&self) -> &Self::Engine;

    async fn get_bucket_location(&self, req: GetBucketLocationRequest) -> Result<GetBucketLocationResponse, Self::Error> {
        let location = self.engine().get_bucket_location(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketLocationResponse { location: Some(location), ..Default::default() })
    }
    async fn get_bucket_policy(&self, req: GetBucketPolicyRequest) -> Result<GetBucketPolicyResponse, Self::Error> {
        let p = self.engine().get_bucket_policy(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketPolicyResponse { json: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_lifecycle(&self, req: GetBucketLifecycleRequest) -> Result<GetBucketLifecycleResponse, Self::Error> {
        let p = self.engine().get_bucket_lifecycle(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketLifecycleResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_encryption(&self, req: GetBucketEncryptionRequest) -> Result<GetBucketEncryptionResponse, Self::Error> {
        let p = self.engine().get_bucket_encryption(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketEncryptionResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_object_lock_config(&self, req: GetBucketObjectLockConfigRequest) -> Result<GetBucketObjectLockConfigResponse, Self::Error> {
        let p = self.engine().get_bucket_object_lock_config(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketObjectLockConfigResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_replication_config(&self, req: GetBucketReplicationConfigRequest) -> Result<GetBucketReplicationConfigResponse, Self::Error> {
        let p = self.engine().get_bucket_replication(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketReplicationConfigResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_versioning(&self, req: GetBucketVersioningRequest) -> Result<GetBucketVersioningResponse, Self::Error> {
        let p = self.engine().get_bucket_versioning(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketVersioningResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_notification(&self, req: GetBucketNotificationRequest) -> Result<GetBucketNotificationResponse, Self::Error> {
        let p = self.engine().get_bucket_notification(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketNotificationResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn listen_bucket_notification(&self, _req: ListenBucketNotificationRequest) -> Result<ListenBucketNotificationResponse, Self::Error> { unsupported("ListenBucketNotification") }
    async fn reset_bucket_replication_status(&self, _req: ResetBucketReplicationStatusRequest) -> Result<ResetBucketReplicationStatusResponse, Self::Error> { unsupported("ResetBucketReplicationStatus") }
    async fn get_bucket_acl(&self, _req: GetBucketAclRequest) -> Result<GetBucketAclResponse, Self::Error> { Ok(Default::default()) }
    async fn put_bucket_acl(&self, _req: PutBucketAclRequest) -> Result<PutBucketAclResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_cors(&self, _req: GetBucketCorsRequest) -> Result<GetBucketCorsResponse, Self::Error> { Ok(Default::default()) }
    async fn put_bucket_cors(&self, _req: PutBucketCorsRequest) -> Result<PutBucketCorsResponse, Self::Error> { Ok(Default::default()) }
    async fn delete_bucket_cors(&self, _req: DeleteBucketCorsRequest) -> Result<DeleteBucketCorsResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_website(&self, _req: GetBucketWebsiteRequest) -> Result<GetBucketWebsiteResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_accelerate(&self, _req: GetBucketAccelerateRequest) -> Result<GetBucketAccelerateResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_request_payment(&self, _req: GetBucketRequestPaymentRequest) -> Result<GetBucketRequestPaymentResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_logging(&self, _req: GetBucketLoggingRequest) -> Result<GetBucketLoggingResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_tagging(&self, req: GetBucketTaggingRequest) -> Result<GetBucketTaggingResponse, Self::Error> {
        let p = self.engine().get_bucket_tagging(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketTaggingResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn delete_bucket_website(&self, _req: DeleteBucketWebsiteRequest) -> Result<DeleteBucketWebsiteResponse, Self::Error> { Ok(Default::default()) }
    async fn delete_bucket_tagging(&self, req: DeleteBucketTaggingRequest) -> Result<DeleteBucketTaggingResponse, Self::Error> {
        self.engine().delete_bucket_tagging(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn list_multipart_uploads(&self, req: ListMultipartUploadsRequest) -> Result<ListMultipartUploadsResponse, Self::Error> {
        let uploads = self.engine().list_multipart_uploads(&req.bucket.bucket, to_list_opt(&req.query, false)).await.map_err(Into::into)?;
        Ok(ListMultipartUploadsResponse {
            uploads: uploads.into_iter().map(|u| MultipartUploadInfo {
                key: u.key,
                upload_id: u.upload_id,
                initiated: Some(u.initiated_at.to_rfc3339()),
            }).collect(),
            ..Default::default()
        })
    }
    async fn list_objects_v2m(&self, req: ListObjectsV2MRequest) -> Result<ListObjectsV2MResponse, Self::Error> {
        let p = self.engine().list_objects_v2(&req.bucket.bucket, to_list_opt(&req.query, req.metadata)).await.map_err(Into::into)?;
        Ok(ListObjectsV2MResponse {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }
    async fn list_objects_v2(&self, req: ListObjectsV2Request) -> Result<ListObjectsV2Response, Self::Error> {
        let p = self.engine().list_objects_v2(&req.bucket.bucket, to_list_opt(&req.query, false)).await.map_err(Into::into)?;
        Ok(ListObjectsV2Response { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
    async fn list_object_versions_m(&self, req: ListObjectVersionsMRequest) -> Result<ListObjectVersionsMResponse, Self::Error> {
        let p = self.engine().list_object_versions(&req.bucket.bucket, to_list_opt(&req.query, req.metadata)).await.map_err(Into::into)?;
        Ok(ListObjectVersionsMResponse { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
    async fn list_object_versions(&self, req: ListObjectVersionsRequest) -> Result<ListObjectVersionsResponse, Self::Error> {
        let p = self.engine().list_object_versions(&req.bucket.bucket, to_list_opt(&req.query, false)).await.map_err(Into::into)?;
        Ok(ListObjectVersionsResponse { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
    async fn get_bucket_policy_status(&self, req: GetBucketPolicyStatusRequest) -> Result<GetBucketPolicyStatusResponse, Self::Error> {
        let p = self.engine().get_bucket_policy_status(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketPolicyStatusResponse { is_public: Some(p.is_public), ..Default::default() })
    }
    async fn put_bucket_lifecycle(&self, req: PutBucketLifecycleRequest) -> Result<PutBucketLifecycleResponse, Self::Error> {
        self.engine().put_bucket_lifecycle(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_replication_config(&self, req: PutBucketReplicationConfigRequest) -> Result<PutBucketReplicationConfigResponse, Self::Error> {
        self.engine().put_bucket_replication(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_encryption(&self, req: PutBucketEncryptionRequest) -> Result<PutBucketEncryptionResponse, Self::Error> {
        self.engine().put_bucket_encryption(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_policy(&self, req: PutBucketPolicyRequest) -> Result<PutBucketPolicyResponse, Self::Error> {
        self.engine().put_bucket_policy(&req.bucket.bucket, req.json).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_object_lock_config(&self, req: PutBucketObjectLockConfigRequest) -> Result<PutBucketObjectLockConfigResponse, Self::Error> {
        self.engine().put_bucket_object_lock_config(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_tagging(&self, req: PutBucketTaggingRequest) -> Result<PutBucketTaggingResponse, Self::Error> {
        self.engine().put_bucket_tagging(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_versioning(&self, req: PutBucketVersioningRequest) -> Result<PutBucketVersioningResponse, Self::Error> {
        self.engine().put_bucket_versioning(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_notification(&self, req: PutBucketNotificationRequest) -> Result<PutBucketNotificationResponse, Self::Error> {
        self.engine().put_bucket_notification(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn reset_bucket_replication_start(&self, _req: ResetBucketReplicationStartRequest) -> Result<ResetBucketReplicationStartResponse, Self::Error> { unsupported("ResetBucketReplicationStart") }
    async fn put_bucket(&self, req: PutBucketRequest) -> Result<PutBucketResponse, Self::Error> {
        let _ = self
            .engine()
            .make_bucket(&req.bucket.bucket, req.region.as_deref(), bucket_features_for_create())
            .await
            .map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn head_bucket(&self, req: HeadBucketRequest) -> Result<HeadBucketResponse, Self::Error> {
        let _ = self.engine().head_bucket(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn post_policy(&self, _req: PostPolicyRequest) -> Result<PostPolicyResponse, Self::Error> { unsupported("PostPolicy") }
    async fn delete_multiple_objects(&self, req: DeleteMultipleObjectsRequest) -> Result<DeleteMultipleObjectsResponse, Self::Error> {
        let keys = parse_delete_keys(&req.payload.xml);
        if keys.is_empty() {
            return Err(S3HandlerBridgeError::InvalidRequest("DeleteMultipleObjects payload has no <Key>".to_string()).into());
        }
        let r = self
            .engine()
            .delete_objects(&req.bucket.bucket, keys, to_delete_opt(None))
            .await
            .map_err(Into::into)?;
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
    async fn delete_bucket_policy(&self, req: DeleteBucketPolicyRequest) -> Result<DeleteBucketPolicyResponse, Self::Error> {
        self.engine().delete_bucket_policy(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn delete_bucket_replication(&self, req: DeleteBucketReplicationRequest) -> Result<DeleteBucketReplicationResponse, Self::Error> {
        self.engine().delete_bucket_replication(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn delete_bucket_lifecycle(&self, req: DeleteBucketLifecycleRequest) -> Result<DeleteBucketLifecycleResponse, Self::Error> {
        self.engine().delete_bucket_lifecycle(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn delete_bucket_encryption(&self, req: DeleteBucketEncryptionRequest) -> Result<DeleteBucketEncryptionResponse, Self::Error> {
        self.engine().delete_bucket_encryption(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn delete_bucket(&self, req: DeleteBucketRequest) -> Result<DeleteBucketResponse, Self::Error> {
        self.engine().delete_bucket(&req.bucket.bucket, false).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn get_bucket_replication_metrics_v2(&self, req: GetBucketReplicationMetricsV2Request) -> Result<GetBucketReplicationMetricsV2Response, Self::Error> {
        let r = self.engine().get_bucket_replication_metrics(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketReplicationMetricsV2Response { json: Some(r.raw_json), ..Default::default() })
    }
    async fn get_bucket_replication_metrics(&self, req: GetBucketReplicationMetricsRequest) -> Result<GetBucketReplicationMetricsResponse, Self::Error> {
        let r = self.engine().get_bucket_replication_metrics(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketReplicationMetricsResponse { json: Some(r.raw_json), ..Default::default() })
    }
    async fn validate_bucket_replication_creds(&self, req: ValidateBucketReplicationCredsRequest) -> Result<ValidateBucketReplicationCredsResponse, Self::Error> {
        let v = self.engine().validate_bucket_replication_creds(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(ValidateBucketReplicationCredsResponse { valid: v.valid, ..Default::default() })
    }
    async fn list_objects_v1(&self, req: ListObjectsV1Request) -> Result<ListObjectsV1Response, Self::Error> {
        let p = self.engine().list_objects_v1(&req.bucket.bucket, to_list_opt(&req.query, false)).await.map_err(Into::into)?;
        Ok(ListObjectsV1Response { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
}
