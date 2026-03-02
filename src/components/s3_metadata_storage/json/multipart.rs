use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::traits::s3_metadata_storage::S3MetadataStorageMultipart;

use super::{JsonMetadataStorage, JsonMetadataStorageError};

#[async_trait]
impl S3MetadataStorageMultipart<JsonMetadataStorageError> for JsonMetadataStorage {
    async fn store_multipart(&self, upload: &MultipartUpload) -> Result<(), JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        if let Some(existing) = snap.multiparts.iter_mut().find(|u| u.upload_id == upload.upload_id) {
            *existing = upload.clone();
        } else {
            snap.multiparts.push(upload.clone());
        }
        self.save_sync(&snap)
    }

    async fn load_multipart(&self, upload_id: &str) -> Result<Option<MultipartUpload>, JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(snap.multiparts.into_iter().find(|u| u.upload_id == upload_id))
    }

    async fn delete_multipart(&self, upload_id: &str) -> Result<(), JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        snap.multiparts.retain(|u| u.upload_id != upload_id);
        snap.multipart_parts.retain(|(id, _)| id != upload_id);
        self.save_sync(&snap)
    }

    async fn store_uploaded_part(&self, upload_id: &str, part: &UploadedPart) -> Result<(), JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        let entry = snap.multipart_parts.iter_mut().find(|(id, _)| id == upload_id);
        match entry {
            Some((_, parts)) => {
                if let Some(existing) = parts.iter_mut().find(|p| p.part_number == part.part_number) {
                    *existing = part.clone();
                } else {
                    parts.push(part.clone());
                }
            }
            None => {
                snap.multipart_parts.push((upload_id.to_owned(), vec![part.clone()]));
            }
        }
        self.save_sync(&snap)
    }

    async fn list_uploaded_parts(&self, upload_id: &str) -> Result<Vec<UploadedPart>, JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(snap.multipart_parts.into_iter()
            .find(|(id, _)| id == upload_id)
            .map(|(_, parts)| parts)
            .unwrap_or_default())
    }

    async fn list_multipart_uploads(&self, bucket: &str) -> Result<Vec<MultipartUpload>, JsonMetadataStorageError> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(snap.multiparts.into_iter().filter(|u| u.bucket == bucket).collect())
    }
}
