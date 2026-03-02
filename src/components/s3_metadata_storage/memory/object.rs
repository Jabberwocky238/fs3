use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::traits::s3_metadata_storage::S3MetadataStorageObject;
use crate::components::s3_engine::S3EngineImplError;

use super::{MemoryMetadataStorage, MemoryMetadataStorageError};

#[async_trait]
impl S3MetadataStorageObject<MemoryMetadataStorageError> for MemoryMetadataStorage {
    async fn store_object_meta(&self, obj: &S3Object) -> Result<(), MemoryMetadataStorageError> {
        let mut state = self.state.write().await;
        state.objects.insert((obj.bucket.clone(), obj.key.clone()), obj.clone());
        Ok(())
    }

    async fn load_object_meta(&self, bucket: &str, key: &str) -> Result<Option<S3Object>, MemoryMetadataStorageError> {
        let state = self.state.read().await;
        Ok(state.objects.get(&(bucket.to_owned(), key.to_owned())).cloned())
    }

    async fn delete_object_meta(&self, bucket: &str, key: &str) -> Result<(), MemoryMetadataStorageError> {
        let mut state = self.state.write().await;
        state.objects.remove(&(bucket.to_owned(), key.to_owned()));
        Ok(())
    }

    async fn list_objects(&self, bucket: &str, options: &ListOptions) -> Result<ObjectListPage, MemoryMetadataStorageError> {
        let state = self.state.read().await;
        let prefix = options.prefix.as_deref().unwrap_or("");
        let max_keys = options.max_keys.unwrap_or(1000) as usize;

        let mut objects: Vec<S3Object> = state.objects.iter()
            .filter(|((b, k), _)| b == bucket && k.starts_with(prefix))
            .map(|(_, v)| v.clone())
            .collect();
        objects.sort_by(|a, b| a.key.cmp(&b.key));

        let is_truncated = objects.len() > max_keys;
        objects.truncate(max_keys);

        Ok(ObjectListPage {
            objects,
            is_truncated,
            ..Default::default()
        })
    }
}

#[async_trait]
impl S3MetadataStorageObject<S3EngineImplError> for MemoryMetadataStorage {
    async fn store_object_meta(&self, obj: &S3Object) -> Result<(), S3EngineImplError> {
        let mut state = self.state.write().await;
        state.objects.insert((obj.bucket.clone(), obj.key.clone()), obj.clone());
        Ok(())
    }
    async fn load_object_meta(&self, bucket: &str, key: &str) -> Result<Option<S3Object>, S3EngineImplError> {
        let state = self.state.read().await;
        Ok(state.objects.get(&(bucket.to_owned(), key.to_owned())).cloned())
    }
    async fn delete_object_meta(&self, bucket: &str, key: &str) -> Result<(), S3EngineImplError> {
        let mut state = self.state.write().await;
        state.objects.remove(&(bucket.to_owned(), key.to_owned()));
        Ok(())
    }
    async fn list_objects(&self, bucket: &str, options: &ListOptions) -> Result<ObjectListPage, S3EngineImplError> {
        let state = self.state.read().await;
        let prefix = options.prefix.as_deref().unwrap_or("");
        let max_keys = options.max_keys.unwrap_or(1000) as usize;
        let mut objects: Vec<S3Object> = state.objects.iter()
            .filter(|((b, k), _)| b == bucket && k.starts_with(prefix))
            .map(|(_, v)| v.clone())
            .collect();
        objects.sort_by(|a, b| a.key.cmp(&b.key));
        let is_truncated = objects.len() > max_keys;
        objects.truncate(max_keys);
        Ok(ObjectListPage { objects, is_truncated, ..Default::default() })
    }
}
