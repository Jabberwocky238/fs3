use async_trait::async_trait;
use std::error::Error;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketNotificationEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketNotificationS3Handler<E: Error + Send + Sync + 'static>: Send + Sync {
    type Engine: S3BucketNotificationEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn bucket_notification_engine_provider(&self) -> &Self::Engine;
    fn bucket_notification_policy_provider(&self) -> &Self::Policy;

    async fn get_bucket_notification(&self, req: GetBucketNotificationRequest) -> Result<GetBucketNotificationResponse, E> {
        check_access(self.bucket_notification_policy_provider(), S3Action::GetBucketNotification, Some(&req.bucket.bucket), None).await?;
        let _p = self.bucket_notification_engine_provider().get_bucket_notification(&req.bucket.bucket).await?;
        Ok(GetBucketNotificationResponse { ..Default::default() })
    }

    async fn put_bucket_notification(&self, req: PutBucketNotificationRequest) -> Result<PutBucketNotificationResponse, E> {
        check_access(self.bucket_notification_policy_provider(), S3Action::PutBucketNotification, Some(&req.bucket.bucket), None).await?;
        let configs = req
            .configs
            .into_iter()
            .map(|config| format!("target={},events={}", config.target_arn, config.events.join("|")))
            .collect();
        self.bucket_notification_engine_provider().put_bucket_notification(&req.bucket.bucket, configs).await?;
        Ok(Default::default())
    }
}
