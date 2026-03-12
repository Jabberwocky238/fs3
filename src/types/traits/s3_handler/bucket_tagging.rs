use async_trait::async_trait;
use crate::types::traits::BoxError;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketTaggingEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;

use super::utils::*;

#[async_trait]
pub trait BucketTaggingS3Handler: Send + Sync {
    type Engine: S3BucketTaggingEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn bucket_tagging_engine_provider(&self) -> &Self::Engine;
    fn bucket_tagging_policy_provider(&self) -> &Self::Policy;

    async fn get_bucket_tagging(&self, req: GetBucketTaggingRequest) -> Result<GetBucketTaggingResponse , BoxError> {
        check_access(self.bucket_tagging_policy_provider(), S3Action::GetBucketTagging, Some(&req.bucket.bucket), None).await?;
        let tags = self.bucket_tagging_engine_provider().get_bucket_tagging(&req.bucket.bucket).await?.unwrap_or_default();
        Ok(GetBucketTaggingResponse { tags, ..Default::default() })
    }

    async fn put_bucket_tagging(&self, req: PutBucketTaggingRequest) -> Result<PutBucketTaggingResponse , BoxError> {
        check_access(self.bucket_tagging_policy_provider(), S3Action::PutBucketTagging, Some(&req.bucket.bucket), None).await?;
        self.bucket_tagging_engine_provider().put_bucket_tagging(&req.bucket.bucket, req.tags).await?;
        Ok(Default::default())
    }

    async fn delete_bucket_tagging(&self, req: DeleteBucketTaggingRequest) -> Result<DeleteBucketTaggingResponse , BoxError> {
        check_access(self.bucket_tagging_policy_provider(), S3Action::PutBucketTagging, Some(&req.bucket.bucket), None).await?;
        self.bucket_tagging_engine_provider().delete_bucket_tagging(&req.bucket.bucket).await?;
        Ok(Default::default())
    }
}

