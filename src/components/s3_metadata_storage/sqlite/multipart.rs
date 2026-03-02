use async_trait::async_trait;

use crate::types::errors::S3MetadataStorageError;
use crate::types::s3::core::*;
use crate::types::traits::s3_metadata_storage::S3MetadataStorageMultipart;

use super::SqliteMetadataStorage;

#[async_trait]
impl S3MetadataStorageMultipart for SqliteMetadataStorage {
    async fn store_multipart(&self, upload: &MultipartUpload) -> Result<(), S3MetadataStorageError> {
        let data = serde_json::to_string(upload)?;
        sqlx::query("INSERT OR REPLACE INTO multiparts (upload_id, bucket, data) VALUES (?, ?, ?)")
            .bind(&upload.upload_id)
            .bind(&upload.bucket)
            .bind(&data)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn load_multipart(&self, upload_id: &str) -> Result<Option<MultipartUpload>, S3MetadataStorageError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT data FROM multiparts WHERE upload_id = ?")
            .bind(upload_id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some((data,)) => Ok(Some(serde_json::from_str(&data)?)),
            None => Ok(None),
        }
    }

    async fn delete_multipart(&self, upload_id: &str) -> Result<(), S3MetadataStorageError> {
        sqlx::query("DELETE FROM multiparts WHERE upload_id = ?")
            .bind(upload_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM multipart_parts WHERE upload_id = ?")
            .bind(upload_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn store_uploaded_part(&self, upload_id: &str, part: &UploadedPart) -> Result<(), S3MetadataStorageError> {
        let data = serde_json::to_string(part)?;
        sqlx::query("INSERT OR REPLACE INTO multipart_parts (upload_id, part_number, data) VALUES (?, ?, ?)")
            .bind(upload_id)
            .bind(part.part_number as i32)
            .bind(&data)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_uploaded_parts(&self, upload_id: &str) -> Result<Vec<UploadedPart>, S3MetadataStorageError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT data FROM multipart_parts WHERE upload_id = ? ORDER BY part_number"
        )
            .bind(upload_id)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|(data,)| serde_json::from_str(&data).map_err(Into::into))
            .collect()
    }

    async fn list_multipart_uploads(&self, bucket: &str) -> Result<Vec<MultipartUpload>, S3MetadataStorageError> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT data FROM multiparts WHERE bucket = ?")
            .bind(bucket)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|(data,)| serde_json::from_str(&data).map_err(Into::into))
            .collect()
    }
}
