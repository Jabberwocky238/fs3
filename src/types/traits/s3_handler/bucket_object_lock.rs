use async_trait::async_trait;
use std::error::Error;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketObjectLockEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketObjectLockS3Handler<E: Error + Send + Sync + 'static>: Send + Sync {
    type Engine: S3BucketObjectLockEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn bucket_object_lock_engine_provider(&self) -> &Self::Engine;
    fn bucket_object_lock_policy_provider(&self) -> &Self::Policy;

    async fn get_bucket_object_lock_config(&self, req: GetBucketObjectLockConfigRequest) -> Result<GetBucketObjectLockConfigResponse, E> {
        check_access(self.bucket_object_lock_policy_provider(), S3Action::GetBucketObjectLockConfiguration, Some(&req.bucket.bucket), None).await?;
        let _p = self.bucket_object_lock_engine_provider().get_bucket_object_lock_config(&req.bucket.bucket).await?;
        Ok(GetBucketObjectLockConfigResponse { ..Default::default() })
    }

    async fn put_bucket_object_lock_config(&self, req: PutBucketObjectLockConfigRequest) -> Result<PutBucketObjectLockConfigResponse, E> {
        check_access(self.bucket_object_lock_policy_provider(), S3Action::PutBucketObjectLockConfiguration, Some(&req.bucket.bucket), None).await?;
        self.bucket_object_lock_engine_provider()
            .put_bucket_object_lock_config(
                &req.bucket.bucket,
                req.config.enabled,
                req.config.mode,
                req.config.days,
                req.config.years,
            )
            .await?;
        Ok(Default::default())
    }
}
