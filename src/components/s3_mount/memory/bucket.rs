use async_trait::async_trait;

use crate::types::errors::S3MountError;
use crate::types::traits::s3_mount::S3MountBucket;

use super::MemoryMount;

#[async_trait]
impl S3MountBucket for MemoryMount {
    async fn create_bucket_dir(&self, bucket: &str) -> Result<(), S3MountError> {
        let mut state = self.state.write().await;
        state.buckets.entry(bucket.to_owned()).or_default();
        Ok(())
    }

    async fn delete_bucket_dir(&self, bucket: &str) -> Result<(), S3MountError> {
        let mut state = self.state.write().await;
        state.buckets.remove(bucket);
        Ok(())
    }

    async fn bucket_dir_exists(&self, bucket: &str) -> Result<bool, S3MountError> {
        let state = self.state.read().await;
        Ok(state.buckets.contains_key(bucket))
    }
}
