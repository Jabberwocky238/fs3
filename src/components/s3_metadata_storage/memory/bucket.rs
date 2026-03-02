use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::errors::S3MetadataStorageError;
use crate::types::traits::s3_metadata_storage::S3MetadataStorageBucket;

use super::MemoryMetadataStorage;

#[async_trait]
impl S3MetadataStorageBucket for MemoryMetadataStorage {
    async fn store_bucket(&self, bucket: &S3Bucket) -> Result<(), S3MetadataStorageError> {
        let mut state = self.state.write().await;
        state.buckets.insert(bucket.identity.name.clone(), bucket.clone());
        Ok(())
    }

    async fn load_bucket(&self, name: &str) -> Result<Option<S3Bucket>, S3MetadataStorageError> {
        let state = self.state.read().await;
        Ok(state.buckets.get(name).cloned())
    }

    async fn list_buckets(&self) -> Result<Vec<S3Bucket>, S3MetadataStorageError> {
        let state = self.state.read().await;
        Ok(state.buckets.values().cloned().collect())
    }

    async fn delete_bucket(&self, name: &str) -> Result<(), S3MetadataStorageError> {
        let mut state = self.state.write().await;
        state.buckets.remove(name);
        state.bucket_metadata.remove(name);
        Ok(())
    }

    async fn store_bucket_metadata(&self, bucket: &str, metadata: &BucketMetadataBundle) -> Result<(), S3MetadataStorageError> {
        let mut state = self.state.write().await;
        state.bucket_metadata.insert(bucket.to_owned(), metadata.clone());
        Ok(())
    }

    async fn load_bucket_metadata(&self, bucket: &str) -> Result<Option<BucketMetadataBundle>, S3MetadataStorageError> {
        let state = self.state.read().await;
        Ok(state.bucket_metadata.get(bucket).cloned())
    }
}
