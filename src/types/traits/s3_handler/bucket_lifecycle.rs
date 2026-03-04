use async_trait::async_trait;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketLifecycleEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketLifecycleS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    fn engine(&self) -> &(impl S3BucketLifecycleEngine + Send + Sync);
    fn policy(&self) -> &impl S3PolicyEngine;

    async fn get_bucket_lifecycle(&self, req: GetBucketLifecycleRequest) -> Result<GetBucketLifecycleResponse, E> {
        check_access(<Self as BucketLifecycleS3Handler<E>>::policy(self), S3Action::GetBucketLifecycle, Some(&req.bucket.bucket), None).await?;
        let _p = <Self as BucketLifecycleS3Handler<E>>::engine(self).get_bucket_lifecycle(&req.bucket.bucket).await?;
        Ok(GetBucketLifecycleResponse { ..Default::default() })
    }

    async fn put_bucket_lifecycle(&self, req: PutBucketLifecycleRequest) -> Result<PutBucketLifecycleResponse, E> {
        check_access(<Self as BucketLifecycleS3Handler<E>>::policy(self), S3Action::PutBucketLifecycle, Some(&req.bucket.bucket), None).await?;
        let rules = parse_lifecycle_rules(&req.xml);
        <Self as BucketLifecycleS3Handler<E>>::engine(self).put_bucket_lifecycle(&req.bucket.bucket, rules).await?;
        Ok(Default::default())
    }

    async fn delete_bucket_lifecycle(&self, req: DeleteBucketLifecycleRequest) -> Result<DeleteBucketLifecycleResponse, E> {
        check_access(<Self as BucketLifecycleS3Handler<E>>::policy(self), S3Action::PutBucketLifecycle, Some(&req.bucket.bucket), None).await?;
        <Self as BucketLifecycleS3Handler<E>>::engine(self).delete_bucket_lifecycle(&req.bucket.bucket).await?;
        Ok(Default::default())
    }
}

fn parse_lifecycle_rules(_xml: &str) -> Vec<String> {
    vec![]
}
