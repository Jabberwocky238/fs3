use async_trait::async_trait;
use crate::types::errors::FS3Error;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketEncryptionEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use super::utils::*;

#[async_trait]
pub trait BucketEncryptionS3Handler: Send + Sync {
    type Engine: S3BucketEncryptionEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn bucket_encryption_engine_provider(&self) -> &Self::Engine;
    fn bucket_encryption_policy_provider(&self) -> &Self::Policy;

    async fn get_bucket_encryption(&self, req: GetBucketEncryptionRequest) -> Result<GetBucketEncryptionResponse , FS3Error> {
        check_access(self.bucket_encryption_policy_provider(), S3Action::GetBucketEncryption, Some(&req.bucket.bucket), None).await?;
        let enc = self.bucket_encryption_engine_provider().get_bucket_encryption(&req.bucket.bucket).await?;
        Ok(GetBucketEncryptionResponse {
            sse_algorithm: enc.as_ref().map(|e| e.algorithm.clone()),
            kms_master_key_id: enc.as_ref().and_then(|e| e.key_id.clone()),
            ..Default::default()
        })
    }

    async fn put_bucket_encryption(&self, req: PutBucketEncryptionRequest) -> Result<PutBucketEncryptionResponse , FS3Error> {
        check_access(self.bucket_encryption_policy_provider(), S3Action::PutBucketEncryption, Some(&req.bucket.bucket), None).await?;
        self.bucket_encryption_engine_provider()
            .put_bucket_encryption(&req.bucket.bucket, req.encryption.algorithm, req.encryption.key_id)
            .await?;
        Ok(Default::default())
    }

    async fn delete_bucket_encryption(&self, req: DeleteBucketEncryptionRequest) -> Result<DeleteBucketEncryptionResponse , FS3Error> {
        check_access(self.bucket_encryption_policy_provider(), S3Action::PutBucketEncryption, Some(&req.bucket.bucket), None).await?;
        self.bucket_encryption_engine_provider().delete_bucket_encryption(&req.bucket.bucket).await?;
        Ok(Default::default())
    }
}

