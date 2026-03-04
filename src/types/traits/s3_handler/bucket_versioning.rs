use async_trait::async_trait;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketVersionEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketVersioningS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    fn engine(&self) -> &(impl S3BucketVersionEngine + Send + Sync);
    fn policy(&self) -> &impl S3PolicyEngine;

    async fn get_bucket_versioning(&self, req: GetBucketVersioningRequest) -> Result<GetBucketVersioningResponse, E> {
        check_access(self.policy(), S3Action::GetBucketVersioning, Some(&req.bucket.bucket), None).await?;
        let _p = self.engine().get_bucket_versioning(&req.bucket.bucket).await?;
        Ok(GetBucketVersioningResponse { ..Default::default() })
    }

    async fn put_bucket_versioning(&self, req: PutBucketVersioningRequest) -> Result<PutBucketVersioningResponse, E> {
        check_access(self.policy(), S3Action::PutBucketVersioning, Some(&req.bucket.bucket), None).await?;
        let (status, mfa_delete) = parse_versioning_config(&req.xml);
        self.engine().put_bucket_versioning(&req.bucket.bucket, status, mfa_delete).await?;
        Ok(Default::default())
    }
}

fn parse_versioning_config(_xml: &str) -> (String, Option<String>) {
    ("Enabled".to_string(), None)
}
