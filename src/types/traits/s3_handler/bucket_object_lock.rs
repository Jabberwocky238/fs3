use async_trait::async_trait;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketObjectLockEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketObjectLockS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    fn engine(&self) -> &(impl S3BucketObjectLockEngine + Send + Sync);
    fn policy(&self) -> &impl S3PolicyEngine;

    async fn get_bucket_object_lock_config(&self, req: GetBucketObjectLockConfigRequest) -> Result<GetBucketObjectLockConfigResponse, E> {
        check_access(self.policy(), S3Action::GetBucketObjectLockConfiguration, Some(&req.bucket.bucket), None).await?;
        let _p = self.engine().get_bucket_object_lock_config(&req.bucket.bucket).await?;
        Ok(GetBucketObjectLockConfigResponse { ..Default::default() })
    }

    async fn put_bucket_object_lock_config(&self, req: PutBucketObjectLockConfigRequest) -> Result<PutBucketObjectLockConfigResponse, E> {
        check_access(self.policy(), S3Action::PutBucketObjectLockConfiguration, Some(&req.bucket.bucket), None).await?;
        let (enabled, mode, days, years) = parse_object_lock_config(&req.xml);
        self.engine().put_bucket_object_lock_config(&req.bucket.bucket, enabled, mode, days, years).await?;
        Ok(Default::default())
    }
}

fn parse_object_lock_config(_xml: &str) -> (bool, Option<String>, Option<u32>, Option<u32>) {
    (false, None, None, None)
}
