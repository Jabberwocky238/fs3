use async_trait::async_trait;
use crate::types::traits::object_layer::ObjectBucketLayer;
use crate::types::s3::object_layer_types::*;
use crate::types::errors::S3Error;
use super::ErasureServerPools;

#[async_trait]
impl ObjectBucketLayer for ErasureServerPools {
    async fn make_bucket(&self, ctx: &Context, bucket: &str, _opts: MakeBucketOptions) -> Result<(), S3Error> {
        self.storage.make_vol(ctx, bucket).await?;
        Ok(())
    }

    async fn get_bucket_info(&self, ctx: &Context, bucket: &str, _opts: BucketOptions) -> Result<BucketInfo, S3Error> {
        let vol = self.storage.stat_vol(ctx, bucket).await
            .map_err(|e| match e {
                crate::types::errors::StorageError::VolumeNotFound(msg) => S3Error::NoSuchBucket(msg),
                _ => S3Error::Storage(e),
            })?;
        Ok(BucketInfo {
            name: vol.name,
            created: vol.created,
        })
    }

    async fn list_buckets(&self, ctx: &Context, _opts: BucketOptions) -> Result<Vec<BucketInfo>, S3Error> {
        let vols = self.storage.list_vols(ctx).await?;
        Ok(vols.into_iter()
            .filter(|v| !v.name.starts_with('.'))
            .map(|v| BucketInfo {
                name: v.name,
                created: v.created,
            }).collect())
    }

    async fn delete_bucket(&self, ctx: &Context, bucket: &str, opts: DeleteBucketOptions) -> Result<(), S3Error> {
        self.storage.delete_vol(ctx, bucket, opts.force).await?;
        Ok(())
    }
}
