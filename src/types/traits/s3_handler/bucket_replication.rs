use async_trait::async_trait;
use std::error::Error;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketReplicationEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketReplicationS3Handler<E: Error + Send + Sync + 'static>: Send + Sync {
    type Engine: S3BucketReplicationEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn bucket_replication_engine_provider(&self) -> &Self::Engine;
    fn bucket_replication_policy_provider(&self) -> &Self::Policy;

    async fn get_bucket_replication_config(&self, req: GetBucketReplicationConfigRequest) -> Result<GetBucketReplicationConfigResponse, E> {
        check_access(self.bucket_replication_policy_provider(), S3Action::GetReplicationConfiguration, Some(&req.bucket.bucket), None).await?;
        let _p = self.bucket_replication_engine_provider().get_bucket_replication(&req.bucket.bucket).await?;
        Ok(GetBucketReplicationConfigResponse { ..Default::default() })
    }

    async fn put_bucket_replication_config(&self, req: PutBucketReplicationConfigRequest) -> Result<PutBucketReplicationConfigResponse, E> {
        check_access(self.bucket_replication_policy_provider(), S3Action::PutReplicationConfiguration, Some(&req.bucket.bucket), None).await?;
        self.bucket_replication_engine_provider()
            .put_bucket_replication(&req.bucket.bucket, req.replication.role, req.replication.rules)
            .await?;
        Ok(Default::default())
    }

    async fn delete_bucket_replication(&self, req: DeleteBucketReplicationRequest) -> Result<DeleteBucketReplicationResponse, E> {
        check_access(self.bucket_replication_policy_provider(), S3Action::PutReplicationConfiguration, Some(&req.bucket.bucket), None).await?;
        self.bucket_replication_engine_provider().delete_bucket_replication(&req.bucket.bucket).await?;
        Ok(Default::default())
    }

    async fn get_bucket_replication_metrics_v2(&self, req: GetBucketReplicationMetricsV2Request) -> Result<GetBucketReplicationMetricsV2Response, E> {
        let _r = self.bucket_replication_engine_provider().get_bucket_replication_metrics(&req.bucket.bucket).await?;
        Ok(GetBucketReplicationMetricsV2Response { ..Default::default() })
    }

    async fn get_bucket_replication_metrics(&self, req: GetBucketReplicationMetricsRequest) -> Result<GetBucketReplicationMetricsResponse, E> {
        let _r = self.bucket_replication_engine_provider().get_bucket_replication_metrics(&req.bucket.bucket).await?;
        Ok(GetBucketReplicationMetricsResponse { ..Default::default() })
    }

    async fn validate_bucket_replication_creds(&self, req: ValidateBucketReplicationCredsRequest) -> Result<ValidateBucketReplicationCredsResponse, E> {
        let v = self.bucket_replication_engine_provider().validate_bucket_replication_creds(&req.bucket.bucket).await?;
        Ok(ValidateBucketReplicationCredsResponse { valid: v.valid, ..Default::default() })
    }
}
