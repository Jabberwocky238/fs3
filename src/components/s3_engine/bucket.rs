use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::errors::S3EngineError;
use crate::types::traits::s3_engine::S3BucketEngine;
use crate::types::traits::s3_metadata_storage::*;
use crate::types::traits::s3_mount::*;

use super::S3EngineImpl;

#[async_trait]
impl<S, M> S3BucketEngine<S3EngineError> for S3EngineImpl<S, M>
where
    S: S3MetadataStorageBucket
        + S3MetadataStorageObject
        + S3MetadataStorageMultipart
        + Send + Sync,
    M: S3MountBucket + Send + Sync,
{
    async fn make_bucket(&self, bucket: &str, region: Option<&str>, features: BucketFeatures) -> Result<S3Bucket, S3EngineError> {
        if self.metadata.load_bucket(bucket).await?.is_some() {
            return Err(S3EngineError::BucketAlreadyExists(bucket.to_owned()));
        }
        let bucket_obj = S3Bucket {
            identity: BucketIdentity {
                name: bucket.to_owned(),
                created_at: chrono::Utc::now(),
                deleted_at: None,
            },
            region: region.map(str::to_owned),
            features,
            tags: TagMap::new(),
        };
        self.metadata.store_bucket(&bucket_obj).await?;
        self.metadata.store_bucket_metadata(bucket, &BucketMetadataBundle::default()).await?;
        self.mount.create_bucket_dir(bucket).await?;
        Ok(bucket_obj)
    }
