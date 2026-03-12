use async_trait::async_trait;
use std::error::Error;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketLifecycleEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketLifecycleS3Handler<E: Error + Send + Sync + 'static>: Send + Sync {
    type Engine: S3BucketLifecycleEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn bucket_lifecycle_engine_provider(&self) -> &Self::Engine;
    fn bucket_lifecycle_policy_provider(&self) -> &Self::Policy;

    async fn get_bucket_lifecycle(&self, req: GetBucketLifecycleRequest) -> Result<GetBucketLifecycleResponse, E> {
        check_access(self.bucket_lifecycle_policy_provider(), S3Action::GetBucketLifecycle, Some(&req.bucket.bucket), None).await?;
        let rules = self.bucket_lifecycle_engine_provider().get_bucket_lifecycle(&req.bucket.bucket).await?;
        Ok(GetBucketLifecycleResponse { rules, ..Default::default() })
    }

    async fn put_bucket_lifecycle(&self, req: PutBucketLifecycleRequest) -> Result<PutBucketLifecycleResponse, E> {
        check_access(self.bucket_lifecycle_policy_provider(), S3Action::PutBucketLifecycle, Some(&req.bucket.bucket), None).await?;
        let rules = req
            .rules
            .into_iter()
            .map(|rule| {
                let mut parts = Vec::new();
                if let Some(id) = rule.id {
                    parts.push(format!("id={id}"));
                }
                if let Some(status) = rule.status {
                    parts.push(format!("status={status}"));
                }
                if let Some(prefix) = rule.prefix {
                    parts.push(format!("prefix={prefix}"));
                }
                parts.join(",")
            })
            .collect();
        self.bucket_lifecycle_engine_provider().put_bucket_lifecycle(&req.bucket.bucket, rules).await?;
        Ok(Default::default())
    }

    async fn delete_bucket_lifecycle(&self, req: DeleteBucketLifecycleRequest) -> Result<DeleteBucketLifecycleResponse, E> {
        check_access(self.bucket_lifecycle_policy_provider(), S3Action::PutBucketLifecycle, Some(&req.bucket.bucket), None).await?;
        self.bucket_lifecycle_engine_provider().delete_bucket_lifecycle(&req.bucket.bucket).await?;
        Ok(Default::default())
    }
}
