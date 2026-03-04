use async_trait::async_trait;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketEncryptionEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketEncryptionS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    fn engine(&self) -> &(impl S3BucketEncryptionEngine + Send + Sync);
    fn policy(&self) -> &impl S3PolicyEngine;

    async fn get_bucket_encryption(&self, req: GetBucketEncryptionRequest) -> Result<GetBucketEncryptionResponse, E> {
        check_access(self.policy(), S3Action::GetBucketEncryption, Some(&req.bucket.bucket), None).await?;
        let _p = self.engine().get_bucket_encryption(&req.bucket.bucket).await?;
        Ok(GetBucketEncryptionResponse { ..Default::default() })
    }

    async fn put_bucket_encryption(&self, req: PutBucketEncryptionRequest) -> Result<PutBucketEncryptionResponse, E> {
        check_access(self.policy(), S3Action::PutBucketEncryption, Some(&req.bucket.bucket), None).await?;
        let (algorithm, key_id) = parse_encryption_config(&req.xml);
        self.engine().put_bucket_encryption(&req.bucket.bucket, algorithm, key_id).await?;
        Ok(Default::default())
    }

    async fn delete_bucket_encryption(&self, req: DeleteBucketEncryptionRequest) -> Result<DeleteBucketEncryptionResponse, E> {
        check_access(self.policy(), S3Action::PutBucketEncryption, Some(&req.bucket.bucket), None).await?;
        self.engine().delete_bucket_encryption(&req.bucket.bucket).await?;
        Ok(Default::default())
    }
}

fn parse_encryption_config(_xml: &str) -> (String, Option<String>) {
    ("AES256".to_string(), None)
}
