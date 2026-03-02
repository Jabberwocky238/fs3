use async_trait::async_trait;

use crate::types::errors::S3MetadataStorageError;
use crate::types::s3::core::*;
use crate::types::traits::s3_metadata_storage::S3MetadataStorageObject;

use super::SqliteMetadataStorage;

#[async_trait]
impl S3MetadataStorageObject for SqliteMetadataStorage {
    async fn store_object_meta(&self, obj: &S3Object) -> Result<(), S3MetadataStorageError> {
        let data = serde_json::to_string(obj)?;
        sqlx::query("INSERT OR REPLACE INTO objects (bucket, key, data) VALUES (?, ?, ?)")
            .bind(&obj.bucket)
            .bind(&obj.key)
            .bind(&data)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn load_object_meta(&self, bucket: &str, key: &str) -> Result<Option<S3Object>, S3MetadataStorageError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT data FROM objects WHERE bucket = ? AND key = ?")
            .bind(bucket)
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some((data,)) => Ok(Some(serde_json::from_str(&data)?)),
            None => Ok(None),
        }
    }

    async fn delete_object_meta(&self, bucket: &str, key: &str) -> Result<(), S3MetadataStorageError> {
        sqlx::query("DELETE FROM objects WHERE bucket = ? AND key = ?")
            .bind(bucket)
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_objects(&self, bucket: &str, options: &ListOptions) -> Result<ObjectListPage, S3MetadataStorageError> {
        let prefix = options.prefix.as_deref().unwrap_or("");
        let max_keys = options.max_keys.unwrap_or(1000) as i32;
        let like_pattern = format!("{}%", prefix);

        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT data FROM objects WHERE bucket = ? AND key LIKE ? ORDER BY key LIMIT ?"
        )
            .bind(bucket)
            .bind(&like_pattern)
            .bind(max_keys + 1)
            .fetch_all(&self.pool)
            .await?;

        let is_truncated = rows.len() > max_keys as usize;
        let objects: Vec<S3Object> = rows.into_iter()
            .take(max_keys as usize)
            .map(|(data,)| serde_json::from_str(&data).map_err(S3MetadataStorageError::from))
            .collect::<Result<_, _>>()?;

        Ok(ObjectListPage {
            objects,
            is_truncated,
            ..Default::default()
        })
    }
}
