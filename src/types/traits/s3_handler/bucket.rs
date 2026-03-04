use async_trait::async_trait;

use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{S3BucketEngine, S3MultipartEngine};
use crate::types::traits::s3_policyengine::{S3PolicyEngine, S3BucketPolicyEngine};
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;

use super::utils::*;

#[async_trait]
pub trait BucketS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>:
    super::BucketLifecycleS3Handler<E>
    + super::BucketEncryptionS3Handler<E>
    + super::BucketObjectLockS3Handler<E>
    + super::BucketVersioningS3Handler<E>
    + super::BucketNotificationS3Handler<E>
    + super::BucketReplicationS3Handler<E>
    + super::BucketTaggingS3Handler<E>
    + Send
    + Sync
where
    <Self as super::BucketLifecycleS3Handler<E>>::Engine: S3BucketEngine + S3MultipartEngine,
    <Self as super::BucketEncryptionS3Handler<E>>::Engine: S3BucketEngine + S3MultipartEngine,
    <Self as super::BucketObjectLockS3Handler<E>>::Engine: S3BucketEngine + S3MultipartEngine,
    <Self as super::BucketVersioningS3Handler<E>>::Engine: S3BucketEngine + S3MultipartEngine,
    <Self as super::BucketNotificationS3Handler<E>>::Engine: S3BucketEngine + S3MultipartEngine,
    <Self as super::BucketReplicationS3Handler<E>>::Engine: S3BucketEngine + S3MultipartEngine,
    <Self as super::BucketTaggingS3Handler<E>>::Engine: S3BucketEngine + S3MultipartEngine,
    <Self as super::BucketLifecycleS3Handler<E>>::Policy: S3PolicyEngine,
    <Self as super::BucketEncryptionS3Handler<E>>::Policy: S3PolicyEngine,
    <Self as super::BucketObjectLockS3Handler<E>>::Policy: S3PolicyEngine,
    <Self as super::BucketVersioningS3Handler<E>>::Policy: S3PolicyEngine,
    <Self as super::BucketNotificationS3Handler<E>>::Policy: S3PolicyEngine,
    <Self as super::BucketReplicationS3Handler<E>>::Policy: S3PolicyEngine,
    <Self as super::BucketTaggingS3Handler<E>>::Policy: S3PolicyEngine,
{
    type Engine: S3BucketEngine + S3MultipartEngine + Send + Sync;
    type Policy: S3PolicyEngine;
    fn engine(&self) -> &Self::Engine;
    fn policy(&self) -> &Self::Policy;

    async fn get_bucket_location(&self, req: GetBucketLocationRequest) -> Result<GetBucketLocationResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::GetBucketLocation, Some(&req.bucket.bucket), None).await?;
        let location = <Self as BucketS3Handler<E>>::engine(self).get_bucket_location(&req.bucket.bucket).await?;
        Ok(GetBucketLocationResponse { location: Some(location), ..Default::default() })
    }

    async fn get_bucket_policy(&self, req: GetBucketPolicyRequest) -> Result<GetBucketPolicyResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::GetBucketPolicy, Some(&req.bucket.bucket), None).await?;
        let p = <Self as BucketS3Handler<E>>::policy(self).get_bucket_policy(&req.bucket.bucket).await
            .map_err(|e| S3EngineError::InvalidPolicy(e.to_string()))?;
        Ok(GetBucketPolicyResponse { config: p.unwrap_or_default(), ..Default::default() })
    }

    async fn put_bucket_policy(&self, req: PutBucketPolicyRequest) -> Result<PutBucketPolicyResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::PutBucketPolicy, Some(&req.bucket.bucket), None).await?;
        <Self as BucketS3Handler<E>>::policy(self).put_bucket_policy(&req.bucket.bucket, &req.json).await
            .map_err(|e| S3EngineError::InvalidPolicy(e.to_string()))?;
        Ok(Default::default())
    }

    async fn delete_bucket_policy(&self, req: DeleteBucketPolicyRequest) -> Result<DeleteBucketPolicyResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::DeleteBucketPolicy, Some(&req.bucket.bucket), None).await?;
        <Self as BucketS3Handler<E>>::policy(self).delete_bucket_policy(&req.bucket.bucket).await
            .map_err(|e| S3EngineError::InvalidPolicy(e.to_string()))?;
        Ok(Default::default())
    }

    async fn get_bucket_policy_status(&self, req: GetBucketPolicyStatusRequest) -> Result<GetBucketPolicyStatusResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::GetBucketPolicyStatus, Some(&req.bucket.bucket), None).await?;
        let p = <Self as BucketS3Handler<E>>::policy(self).get_bucket_policy(&req.bucket.bucket).await
            .map_err(|e| S3EngineError::InvalidPolicy(e.to_string()))?;
        let is_public = p.as_ref()
            .map(|d| d.to_ascii_lowercase().contains("\"effect\":\"allow\""))
            .unwrap_or(false);
        Ok(GetBucketPolicyStatusResponse { is_public: Some(is_public), ..Default::default() })
    }

    async fn listen_bucket_notification(&self, _req: ListenBucketNotificationRequest) -> Result<ListenBucketNotificationResponse, E> {
        unsupported("ListenBucketNotification")
    }

    async fn reset_bucket_replication_status(&self, _req: ResetBucketReplicationStatusRequest) -> Result<ResetBucketReplicationStatusResponse, E> {
        unsupported("ResetBucketReplicationStatus")
    }

    async fn reset_bucket_replication_start(&self, _req: ResetBucketReplicationStartRequest) -> Result<ResetBucketReplicationStartResponse, E> {
        unsupported("ResetBucketReplicationStart")
    }

    async fn get_bucket_acl(&self, _req: GetBucketAclRequest) -> Result<GetBucketAclResponse, E> {
        Ok(Default::default())
    }

    async fn put_bucket_acl(&self, _req: PutBucketAclRequest) -> Result<PutBucketAclResponse, E> {
        Ok(Default::default())
    }

    async fn get_bucket_cors(&self, _req: GetBucketCorsRequest) -> Result<GetBucketCorsResponse, E> {
        Ok(Default::default())
    }

    async fn put_bucket_cors(&self, _req: PutBucketCorsRequest) -> Result<PutBucketCorsResponse, E> {
        Ok(Default::default())
    }

    async fn delete_bucket_cors(&self, _req: DeleteBucketCorsRequest) -> Result<DeleteBucketCorsResponse, E> {
        Ok(Default::default())
    }

    async fn get_bucket_website(&self, _req: GetBucketWebsiteRequest) -> Result<GetBucketWebsiteResponse, E> {
        Ok(Default::default())
    }

    async fn delete_bucket_website(&self, _req: DeleteBucketWebsiteRequest) -> Result<DeleteBucketWebsiteResponse, E> {
        Ok(Default::default())
    }

    async fn get_bucket_accelerate(&self, _req: GetBucketAccelerateRequest) -> Result<GetBucketAccelerateResponse, E> {
        Ok(Default::default())
    }

    async fn get_bucket_request_payment(&self, _req: GetBucketRequestPaymentRequest) -> Result<GetBucketRequestPaymentResponse, E> {
        Ok(Default::default())
    }

    async fn get_bucket_logging(&self, _req: GetBucketLoggingRequest) -> Result<GetBucketLoggingResponse, E> {
        Ok(Default::default())
    }

    async fn list_multipart_uploads(&self, req: ListMultipartUploadsRequest) -> Result<ListMultipartUploadsResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::ListBucketMultipartUploads, Some(&req.bucket.bucket), None).await?;
        let uploads = <Self as BucketS3Handler<E>>::engine(self).list_multipart_uploads(&req.bucket.bucket, to_list_opt(&req.query, false)).await?;
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
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::ListBucket, Some(&req.bucket.bucket), None).await?;
        let p = <Self as BucketS3Handler<E>>::engine(self).list_objects_v2(&req.bucket.bucket, to_list_opt(&req.query, req.metadata)).await?;
        Ok(ListObjectsV2MResponse {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }

    async fn list_objects_v2(&self, req: ListObjectsV2Request) -> Result<ListObjectsV2Response, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::ListBucket, Some(&req.bucket.bucket), None).await?;
        let p = <Self as BucketS3Handler<E>>::engine(self).list_objects_v2(&req.bucket.bucket, to_list_opt(&req.query, false)).await?;
        Ok(ListObjectsV2Response {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }

    async fn list_object_versions_m(&self, req: ListObjectVersionsMRequest) -> Result<ListObjectVersionsMResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::ListBucketVersions, Some(&req.bucket.bucket), None).await?;
        let p = <Self as BucketS3Handler<E>>::engine(self).list_object_versions(&req.bucket.bucket, to_list_opt(&req.query, req.metadata)).await?;
        Ok(ListObjectVersionsMResponse {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }

    async fn list_object_versions(&self, req: ListObjectVersionsRequest) -> Result<ListObjectVersionsResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::ListBucketVersions, Some(&req.bucket.bucket), None).await?;
        let p = <Self as BucketS3Handler<E>>::engine(self).list_object_versions(&req.bucket.bucket, to_list_opt(&req.query, false)).await?;
        Ok(ListObjectVersionsResponse {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }

    async fn list_objects_v1(&self, req: ListObjectsV1Request) -> Result<ListObjectsV1Response, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::ListBucket, Some(&req.bucket.bucket), None).await?;
        let p = <Self as BucketS3Handler<E>>::engine(self).list_objects_v1(&req.bucket.bucket, to_list_opt(&req.query, false)).await?;
        Ok(ListObjectsV1Response {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }

    async fn put_bucket(&self, req: PutBucketRequest) -> Result<PutBucketResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::CreateBucket, Some(&req.bucket.bucket), None).await?;
        let _ = <Self as BucketS3Handler<E>>::engine(self)
            .make_bucket(&req.bucket.bucket, req.region.as_deref(), bucket_features_for_create())
            .await?;
        Ok(Default::default())
    }

    async fn head_bucket(&self, req: HeadBucketRequest) -> Result<HeadBucketResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::HeadBucket, Some(&req.bucket.bucket), None).await?;
        let _ = <Self as BucketS3Handler<E>>::engine(self).head_bucket(&req.bucket.bucket).await?;
        Ok(Default::default())
    }

    async fn post_policy(&self, _req: PostPolicyRequest) -> Result<PostPolicyResponse, E> {
        unsupported("PostPolicy")
    }

    async fn delete_multiple_objects(&self, req: DeleteMultipleObjectsRequest) -> Result<DeleteMultipleObjectsResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::DeleteObject, Some(&req.bucket.bucket), None).await?;
        let keys = parse_delete_keys(&req.payload.xml);
        if keys.is_empty() {
            return Err(S3HandlerBridgeError::InvalidRequest("DeleteMultipleObjects payload has no <Key>".to_string()).into());
        }
        let r = <Self as BucketS3Handler<E>>::engine(self)
            .delete_objects(&req.bucket.bucket, keys, to_delete_opt(None))
            .await?;
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

    async fn delete_bucket(&self, req: DeleteBucketRequest) -> Result<DeleteBucketResponse, E> {
        check_access(<Self as BucketS3Handler<E>>::policy(self), S3Action::DeleteBucket, Some(&req.bucket.bucket), None).await?;
        <Self as BucketS3Handler<E>>::engine(self).delete_bucket(&req.bucket.bucket, false).await?;
        Ok(Default::default())
    }
}
