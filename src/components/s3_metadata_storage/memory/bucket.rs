use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::traits::s3_metadata_storage::S3MetadataStorageBucket;
use crate::components::s3_engine::S3EngineImplError;

use super::{MemoryMetadataStorage, MemoryMetadataStorageError};

#[async_trait]
impl S3MetadataStorageBucket<MemoryMetadataStorageError> for MemoryMetadataStorage {
    async fn store_bucket(&self, bucket: &S3Bucket) -> Result<(), MemoryMetadataStorageError> {
        let mut state = self.state.write().await;
        state.buckets.insert(bucket.identity.name.clone(), bucket.clone());
        Ok(())
    }

    async fn load_bucket(&self, name: &str) -> Result<Option<S3Bucket>, MemoryMetadataStorageError> {
        let state = self.state.read().await;
        Ok(state.buckets.get(name).cloned())
    }

    async fn list_buckets(&self) -> Result<Vec<S3Bucket>, MemoryMetadataStorageError> {
        let state = self.state.read().await;
        Ok(state.buckets.values().cloned().collect())
    }

    async fn delete_bucket(&self, name: &str) -> Result<(), MemoryMetadataStorageError> {
        let mut state = self.state.write().await;
        state.buckets.remove(name);
        state.bucket_metadata.remove(name);
        Ok(())
    }

    async fn store_bucket_metadata(&self, bucket: &str, metadata: &BucketMetadataBundle) -> Result<(), MemoryMetadataStorageError> {
        let mut state = self.state.write().await;
        state.bucket_metadata.insert(bucket.to_owned(), metadata.clone());
        Ok(())
    }

    async fn load_bucket_metadata(&self, bucket: &str) -> Result<Option<BucketMetadataBundle>, MemoryMetadataStorageError> {
        let state = self.state.read().await;
        Ok(state.bucket_metadata.get(bucket).cloned())
    }
}

#[async_trait]
impl S3MetadataStorageBucket<S3EngineImplError> for MemoryMetadataStorage {
    async fn store_bucket(&self, bucket: &S3Bucket) -> Result<(), S3EngineImplError> {
        let mut state = self.state.write().await;
        state.buckets.insert(bucket.identity.name.clone(), bucket.clone());
        Ok(())
    }
    async fn load_bucket(&self, name: &str) -> Result<Option<S3Bucket>, S3EngineImplError> {
        let state = self.state.read().await;
        Ok(state.buckets.get(name).cloned())
    }
    async fn list_buckets(&self) -> Result<Vec<S3Bucket>, S3EngineImplError> {
        let state = self.state.read().await;
        Ok(state.buckets.values().cloned().collect())
    }
    async fn delete_bucket(&self, name: &str) -> Result<(), S3EngineImplError> {
        let mut state = self.state.write().await;
        state.buckets.remove(name);
        state.bucket_metadata.remove(name);
        Ok(())
    }
    async fn store_bucket_metadata(&self, bucket: &str, metadata: &BucketMetadataBundle) -> Result<(), S3EngineImplError> {
        let mut state = self.state.write().await;
        state.bucket_metadata.insert(bucket.to_owned(), metadata.clone());
        Ok(())
    }
    async fn load_bucket_metadata(&self, bucket: &str) -> Result<Option<BucketMetadataBundle>, S3EngineImplError> {
        let state = self.state.read().await;
        Ok(state.bucket_metadata.get(bucket).cloned())
    }
}
