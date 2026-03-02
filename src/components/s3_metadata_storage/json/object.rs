use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::s3::metadata_storage_error::S3MetadataStorageError;
use crate::types::traits::s3_metadata_storage::S3MetadataStorageObject;

use super::JsonMetadataStorage;

#[async_trait]
impl S3MetadataStorageObject for JsonMetadataStorage {
    async fn store_object_meta(&self, obj: &S3Object) -> Result<(), S3MetadataStorageError> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        if let Some(existing) = snap.objects.iter_mut().find(|o| o.bucket == obj.bucket && o.key == obj.key) {
            *existing = obj.clone();
        } else {
            snap.objects.push(obj.clone());
        }
        self.save_sync(&snap)
    }

    async fn load_object_meta(&self, bucket: &str, key: &str) -> Result<Option<S3Object>, S3MetadataStorageError> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(snap.objects.into_iter().find(|o| o.bucket == bucket && o.key == key))
    }

    async fn delete_object_meta(&self, bucket: &str, key: &str) -> Result<(), S3MetadataStorageError> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        snap.objects.retain(|o| !(o.bucket == bucket && o.key == key));
        self.save_sync(&snap)
    }

    async fn list_objects(&self, bucket: &str, options: &ListOptions) -> Result<ObjectListPage, S3MetadataStorageError> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        let prefix = options.prefix.as_deref().unwrap_or("");
        let max_keys = options.max_keys.unwrap_or(1000) as usize;

        let mut objects: Vec<S3Object> = snap.objects.into_iter()
            .filter(|o| o.bucket == bucket && o.key.starts_with(prefix))
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
