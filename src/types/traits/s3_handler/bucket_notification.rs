use async_trait::async_trait;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketNotificationEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketNotificationS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    fn engine(&self) -> &(impl S3BucketNotificationEngine + Send + Sync);
    fn policy(&self) -> &impl S3PolicyEngine;

    async fn get_bucket_notification(&self, req: GetBucketNotificationRequest) -> Result<GetBucketNotificationResponse, E> {
        check_access(self.policy(), S3Action::GetBucketNotification, Some(&req.bucket.bucket), None).await?;
        let _p = self.engine().get_bucket_notification(&req.bucket.bucket).await?;
        Ok(GetBucketNotificationResponse { ..Default::default() })
    }

    async fn put_bucket_notification(&self, req: PutBucketNotificationRequest) -> Result<PutBucketNotificationResponse, E> {
        check_access(self.policy(), S3Action::PutBucketNotification, Some(&req.bucket.bucket), None).await?;
        let configs = parse_notification_config(&req.xml);
        self.engine().put_bucket_notification(&req.bucket.bucket, configs).await?;
        Ok(Default::default())
    }
}

fn parse_notification_config(_xml: &str) -> Vec<String> {
    vec![]
}
