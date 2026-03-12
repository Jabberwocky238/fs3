use async_trait::async_trait;
use std::error::Error;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketVersionEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketVersioningS3Handler<E: Error + Send + Sync + 'static>: Send + Sync {
    type Engine: S3BucketVersionEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn bucket_versioning_engine_provider(&self) -> &Self::Engine;
    fn bucket_versioning_policy_provider(&self) -> &Self::Policy;

    async fn get_bucket_versioning(&self, req: GetBucketVersioningRequest) -> Result<GetBucketVersioningResponse, E> {
        check_access(self.bucket_versioning_policy_provider(), S3Action::GetBucketVersioning, Some(&req.bucket.bucket), None).await?;
        let v = self.bucket_versioning_engine_provider().get_bucket_versioning(&req.bucket.bucket).await?;
        Ok(GetBucketVersioningResponse {
            status: v.as_ref().map(|x| x.status.clone()),
            mfa_delete: v.as_ref().and_then(|x| x.mfa_delete.clone()),
            ..Default::default()
        })
    }

    async fn put_bucket_versioning(&self, req: PutBucketVersioningRequest) -> Result<PutBucketVersioningResponse, E> {
        check_access(self.bucket_versioning_policy_provider(), S3Action::PutBucketVersioning, Some(&req.bucket.bucket), None).await?;
        self.bucket_versioning_engine_provider()
            .put_bucket_versioning(&req.bucket.bucket, req.versioning.status, req.versioning.mfa_delete)
            .await?;
        Ok(Default::default())
    }
}
