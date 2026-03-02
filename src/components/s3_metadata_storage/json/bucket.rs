use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::traits::s3_metadata_storage::S3MetadataStorageBucket;

use super::{JsonMetadataStorage, JsonMetadataStorageError};

#[async_trait]
impl S3MetadataStorageBucket<JsonMetadataStorageError> for JsonMetadataStorage {
    async fn store_bucket(&self, bucket: &S3Bucket) -> Result<(), JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        if let Some(existing) = snap.buckets.iter_mut().find(|b| b.identity.name == bucket.identity.name) {
            *existing = bucket.clone();
        } else {
            snap.buckets.push(bucket.clone());
        }
        self.save_sync(&snap)
    }

    async fn load_bucket(&self, name: &str) -> Result<Option<S3Bucket>, JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(snap.buckets.into_iter().find(|b| b.identity.name == name))
    }

    async fn list_buckets(&self) -> Result<Vec<S3Bucket>, JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(snap.buckets)
    }

    async fn delete_bucket(&self, name: &str) -> Result<(), JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        snap.buckets.retain(|b| b.identity.name != name);
        snap.bucket_metadata.retain(|(n, _)| n != name);
        self.save_sync(&snap)
    }

    async fn store_bucket_metadata(&self, bucket: &str, metadata: &BucketMetadataBundle) -> Result<(), JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        if let Some(existing) = snap.bucket_metadata.iter_mut().find(|(n, _)| n == bucket) {
            existing.1 = metadata.clone();
        } else {
            snap.bucket_metadata.push((bucket.to_owned(), metadata.clone()));
        }
        self.save_sync(&snap)
    }

    async fn load_bucket_metadata(&self, bucket: &str) -> Result<Option<BucketMetadataBundle>, JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(snap.bucket_metadata.into_iter().find(|(n, _)| n == bucket).map(|(_, m)| m))
    }
}
