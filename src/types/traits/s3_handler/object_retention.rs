use crate::types::s3::policy::S3Action;
use crate::types::s3::request::*;
use crate::types::s3::response::*;

use crate::types::traits::s3_engine::S3ObjectRetentionEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use async_trait::async_trait;

use super::utils::*;

#[async_trait]
pub trait ObjectRetentionS3Handler: Send + Sync {
    type Engine: S3ObjectRetentionEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn object_retention_engine_provider(&self) -> &Self::Engine;
    fn object_retention_policy_provider(&self) -> &Self::Policy;

    async fn get_object_retention(
        &self,
        req: GetObjectRetentionRequest,
    ) -> Result<GetObjectRetentionResponse, BoxError> {
        check_access(
            self.object_retention_policy_provider(),
            S3Action::GetObjectRetention,
            Some(&req.bucket.bucket),
            Some(&req.object.object),
        )
        .await?;
        let _r = self
            .object_retention_engine_provider()
            .get_object_retention(&req.bucket.bucket, &req.object.object)
            .await?;
        Ok(GetObjectRetentionResponse {
            ..Default::default()
        })
    }

    async fn put_object_retention(
        &self,
        req: PutObjectRetentionRequest,
    ) -> Result<PutObjectRetentionResponse, BoxError> {
        check_access(
            self.object_retention_policy_provider(),
            S3Action::PutObjectRetention,
            Some(&req.bucket.bucket),
            Some(&req.object.object),
        )
        .await?;
        self.object_retention_engine_provider()
            .put_object_retention(&req.bucket.bucket, &req.object.object, req.retention)
            .await?;
        Ok(Default::default())
    }
}
