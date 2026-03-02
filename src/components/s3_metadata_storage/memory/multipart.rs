use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::errors::S3MetadataStorageError;
use crate::types::traits::s3_metadata_storage::S3MetadataStorageMultipart;

use super::MemoryMetadataStorage;

#[async_trait]
impl S3MetadataStorageMultipart for MemoryMetadataStorage {
    async fn store_multipart(&self, upload: &MultipartUpload) -> Result<(), S3MetadataStorageError> {
        let mut state = self.state.write().await;
        state.multiparts.insert(upload.upload_id.clone(), upload.clone());
        Ok(())
    }

    async fn load_multipart(&self, upload_id: &str) -> Result<Option<MultipartUpload>, S3MetadataStorageError> {
        let state = self.state.read().await;
        Ok(state.multiparts.get(upload_id).cloned())
    }

    async fn delete_multipart(&self, upload_id: &str) -> Result<(), S3MetadataStorageError> {
        let mut state = self.state.write().await;
        state.multiparts.remove(upload_id);
        state.multipart_parts.remove(upload_id);
        Ok(())
    }

    async fn store_uploaded_part(&self, upload_id: &str, part: &UploadedPart) -> Result<(), S3MetadataStorageError> {
        let mut state = self.state.write().await;
        let parts = state.multipart_parts.entry(upload_id.to_owned()).or_default();
        if let Some(existing) = parts.iter_mut().find(|p| p.part_number == part.part_number) {
            *existing = part.clone();
        } else {
            parts.push(part.clone());
        }
        Ok(())
    }

    async fn list_uploaded_parts(&self, upload_id: &str) -> Result<Vec<UploadedPart>, S3MetadataStorageError> {
        let state = self.state.read().await;
        Ok(state.multipart_parts.get(upload_id).cloned().unwrap_or_default())
    }

    async fn list_multipart_uploads(&self, bucket: &str) -> Result<Vec<MultipartUpload>, S3MetadataStorageError> {
        let state = self.state.read().await;
        Ok(state.multiparts.values()
            .filter(|u| u.bucket == bucket)
            .cloned()
            .collect())
    }
}
