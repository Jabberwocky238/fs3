use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::errors::S3EngineError;
use crate::types::traits::s3_engine::S3BucketEngine;
use crate::types::traits::s3_metadata_storage::*;
use crate::types::traits::s3_mount::*;

use super::S3EngineImpl;

#[async_trait]
impl<S, M> S3BucketEngine for S3EngineImpl<S, M>
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

    async fn head_bucket(&self, bucket: &str) -> Result<S3Bucket, S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))
    }

    async fn get_bucket(&self, bucket: &str) -> Result<S3Bucket, S3EngineError> {
        self.head_bucket(bucket).await
    }

    async fn list_buckets(&self) -> Result<Vec<S3Bucket>, S3EngineError> {
        let mut out = self.metadata.list_buckets().await?;
        out.sort_by(|a, b| a.identity.name.cmp(&b.identity.name));
        Ok(out)
    }

    async fn delete_bucket(&self, bucket: &str, force: bool) -> Result<(), S3EngineError> {
        self.head_bucket(bucket).await?;
        let page = self.metadata.list_objects(bucket, &ListOptions { max_keys: Some(1), ..Default::default() }).await?;
        if !page.objects.is_empty() && !force {
            return Err(S3EngineError::BucketNotEmpty(bucket.to_owned()));
        }
        if force {
            let all = self.metadata.list_objects(bucket, &ListOptions::default()).await?;
            for obj in &all.objects {
                self.metadata.delete_object_meta(bucket, &obj.key).await?;
            }
            let uploads = self.metadata.list_multipart_uploads(bucket).await?;
            for u in &uploads {
                self.metadata.delete_multipart(&u.upload_id).await?;
            }
        }
        self.metadata.delete_bucket(bucket).await?;
        self.mount.delete_bucket_dir(bucket).await?;
        Ok(())
    }

    async fn list_objects_v1(&self, bucket: &str, options: ListOptions) -> Result<ObjectListPage, S3EngineError> {
        self.head_bucket(bucket).await?;
        self.metadata.list_objects(bucket, &options).await
    }

    async fn list_objects_v2(&self, bucket: &str, options: ListOptions) -> Result<ObjectListPage, S3EngineError> {
        self.head_bucket(bucket).await?;
        self.metadata.list_objects(bucket, &options).await
    }

    async fn list_object_versions(&self, bucket: &str, options: ListOptions) -> Result<ObjectListPage, S3EngineError> {
        self.head_bucket(bucket).await?;
        self.metadata.list_objects(bucket, &options).await
    }
}
