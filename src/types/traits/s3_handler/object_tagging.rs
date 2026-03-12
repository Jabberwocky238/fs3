use async_trait::async_trait;

use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3ObjectTaggingEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;

use super::utils::*;

#[async_trait]
pub trait ObjectTaggingS3Handler: Send + Sync {
    type Engine: S3ObjectTaggingEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn object_tagging_engine_provider(&self) -> &Self::Engine;
    fn object_tagging_policy_provider(&self) -> &Self::Policy;

    async fn get_object_tagging(&self, req: GetObjectTaggingRequest) -> Result<GetObjectTaggingResponse , BoxError> {
        check_access(self.object_tagging_policy_provider(), S3Action::GetObjectTagging, Some(&req.object.bucket), Some(&req.object.object)).await?;
        let tags = self.object_tagging_engine_provider().get_object_tagging(&req.object.bucket, &req.object.object).await?;
        Ok(GetObjectTaggingResponse { tags, ..Default::default() })
    }

    async fn put_object_tagging(&self, req: PutObjectTaggingRequest) -> Result<PutObjectTaggingResponse , BoxError> {
        check_access(self.object_tagging_policy_provider(), S3Action::PutObjectTagging, Some(&req.object.bucket), Some(&req.object.object)).await?;
        self.object_tagging_engine_provider().put_object_tagging(&req.object.bucket, &req.object.object, req.tags).await?;
        Ok(Default::default())
    }

    async fn delete_object_tagging(&self, req: DeleteObjectTaggingRequest) -> Result<DeleteObjectTaggingResponse , BoxError> {
        check_access(self.object_tagging_policy_provider(), S3Action::DeleteObjectTagging, Some(&req.object.bucket), Some(&req.object.object)).await?;
        self.object_tagging_engine_provider().delete_object_tagging(&req.object.bucket, &req.object.object).await?;
        Ok(Default::default())
    }
}

