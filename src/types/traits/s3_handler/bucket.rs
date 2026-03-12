use crate::types::traits::BoxError;
use async_trait::async_trait;

use crate::types::s3::policy::S3Action;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{
    S3BucketConfigEngine, S3BucketEngine, S3MultipartEngine, S3ObjectEngine,
};
use crate::types::traits::s3_policyengine::{S3BucketPolicyEngine, S3PolicyEngine};

use super::utils::*;

#[async_trait]
pub trait BucketS3Handler: Send + Sync {
    type Engine: S3BucketEngine + S3MultipartEngine + S3ObjectEngine + Send + Sync;
    type Policy: S3PolicyEngine;
    fn engine(&self) -> &Self::Engine;
    fn policy(&self) -> &Self::Policy;

    async fn get_bucket_location(
        &self,
        req: GetBucketLocationRequest,
    ) -> Result<GetBucketLocationResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::GetBucketLocation,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        let location = self
            .engine()
            .get_bucket_location(&req.bucket.bucket)
            .await?;
        Ok(GetBucketLocationResponse {
            location: Some(location),
            ..Default::default()
        })
    }

    async fn get_bucket_policy(
        &self,
        req: GetBucketPolicyRequest,
    ) -> Result<GetBucketPolicyResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::GetBucketPolicy,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        let p = self
            .policy()
            .get_bucket_policy(&req.bucket.bucket)
            .await?;
        Ok(GetBucketPolicyResponse {
            config: p.unwrap_or_default(),
            ..Default::default()
        })
    }

    async fn put_bucket_policy(
        &self,
        req: PutBucketPolicyRequest,
    ) -> Result<PutBucketPolicyResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::PutBucketPolicy,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        self.policy()
            .put_bucket_policy(&req.bucket.bucket, &req.json)
            .await?;
        Ok(Default::default())
    }

    async fn delete_bucket_policy(
        &self,
        req: DeleteBucketPolicyRequest,
    ) -> Result<DeleteBucketPolicyResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::DeleteBucketPolicy,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        self.policy()
            .delete_bucket_policy(&req.bucket.bucket)
            .await?;
        Ok(Default::default())
    }

    async fn get_bucket_policy_status(
        &self,
        req: GetBucketPolicyStatusRequest,
    ) -> Result<GetBucketPolicyStatusResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::GetBucketPolicyStatus,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        let p = self
            .policy()
            .get_bucket_policy(&req.bucket.bucket)
            .await?;
        let is_public = p
            .as_ref()
            .map(|d| d.to_ascii_lowercase().contains("\"effect\":\"allow\""))
            .unwrap_or(false);
        Ok(GetBucketPolicyStatusResponse {
            is_public: Some(is_public),
            ..Default::default()
        })
    }

    async fn list_multipart_uploads(
        &self,
        req: ListMultipartUploadsRequest,
    ) -> Result<ListMultipartUploadsResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::ListBucketMultipartUploads,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        let uploads = self
            .engine()
            .list_multipart_uploads(&req.bucket.bucket, to_list_opt(&req.query, false))
            .await?;
        Ok(ListMultipartUploadsResponse {
            uploads: uploads
                .into_iter()
                .map(|u| MultipartUploadInfo {
                    key: u.key,
                    upload_id: u.upload_id,
                    initiated: Some(u.initiated_at.to_rfc3339()),
                })
                .collect(),
            ..Default::default()
        })
    }

    async fn list_objects_v2m(
        &self,
        req: ListObjectsV2MRequest,
    ) -> Result<ListObjectsV2MResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::ListBucket,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        let p = self
            .engine()
            .list_objects_v2(&req.bucket.bucket, to_list_opt(&req.query, req.metadata))
            .await?;
        Ok(ListObjectsV2MResponse {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }

    async fn list_objects_v2(
        &self,
        req: ListObjectsV2Request,
    ) -> Result<ListObjectsV2Response, BoxError> {
        check_access(
            self.policy(),
            S3Action::ListBucket,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        let p = self
            .engine()
            .list_objects_v2(&req.bucket.bucket, to_list_opt(&req.query, false))
            .await?;
        Ok(ListObjectsV2Response {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }

    async fn list_object_versions_m(
        &self,
        req: ListObjectVersionsMRequest,
    ) -> Result<ListObjectVersionsMResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::ListBucketVersions,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        let p = self
            .engine()
            .list_object_versions(&req.bucket.bucket, to_list_opt(&req.query, req.metadata))
            .await?;
        Ok(ListObjectVersionsMResponse {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }

    async fn list_object_versions(
        &self,
        req: ListObjectVersionsRequest,
    ) -> Result<ListObjectVersionsResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::ListBucketVersions,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        let p = self
            .engine()
            .list_object_versions(&req.bucket.bucket, to_list_opt(&req.query, false))
            .await?;
        Ok(ListObjectVersionsResponse {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }

    async fn list_objects_v1(
        &self,
        req: ListObjectsV1Request,
    ) -> Result<ListObjectsV1Response, BoxError> {
        check_access(
            self.policy(),
            S3Action::ListBucket,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        let p = self
            .engine()
            .list_objects_v1(&req.bucket.bucket, to_list_opt(&req.query, false))
            .await?;
        Ok(ListObjectsV1Response {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }

    async fn put_bucket(&self, req: PutBucketRequest) -> Result<PutBucketResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::CreateBucket,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        let _ = self
            .engine()
            .make_bucket(
                &req.bucket.bucket,
                req.region.as_deref(),
                bucket_features_for_create(),
            )
            .await?;
        Ok(Default::default())
    }

    async fn head_bucket(&self, req: HeadBucketRequest) -> Result<HeadBucketResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::HeadBucket,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        let _ = self.engine().head_bucket(&req.bucket.bucket).await?;
        Ok(Default::default())
    }

    async fn delete_multiple_objects(
        &self,
        req: DeleteMultipleObjectsRequest,
    ) -> Result<DeleteMultipleObjectsResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::DeleteObject,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        if req.payload.keys.is_empty() {
            return Err(S3HandlerBridgeError::InvalidRequest(
                "DeleteMultipleObjects payload has no <Key>".to_string(),
            )
            .into());
        }
        let r = self
            .engine()
            .delete_objects(&req.bucket.bucket, req.payload.keys, to_delete_opt(None))
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

    async fn delete_bucket(
        &self,
        req: DeleteBucketRequest,
    ) -> Result<DeleteBucketResponse, BoxError> {
        check_access(
            self.policy(),
            S3Action::DeleteBucket,
            Some(&req.bucket.bucket),
            None,
        )
        .await?;
        self.engine()
            .delete_bucket(&req.bucket.bucket, false)
            .await?;
        Ok(Default::default())
    }
}
