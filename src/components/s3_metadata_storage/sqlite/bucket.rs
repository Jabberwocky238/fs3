use async_trait::async_trait;

use crate::types::errors::S3MetadataStorageError;
use crate::types::s3::core::*;
use crate::types::traits::s3_metadata_storage::S3MetadataStorageBucket;

use super::SqliteMetadataStorage;

#[async_trait]
impl S3MetadataStorageBucket for SqliteMetadataStorage {
    async fn store_bucket(&self, bucket: &S3Bucket) -> Result<(), S3MetadataStorageError> {
        let data = serde_json::to_string(bucket)?;
        sqlx::query("INSERT OR REPLACE INTO buckets (name, data) VALUES (?, ?)")
            .bind(&bucket.identity.name)
            .bind(&data)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn load_bucket(&self, name: &str) -> Result<Option<S3Bucket>, S3MetadataStorageError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT data FROM buckets WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some((data,)) => Ok(Some(serde_json::from_str(&data)?)),
            None => Ok(None),
        }
    }

    async fn list_buckets(&self) -> Result<Vec<S3Bucket>, S3MetadataStorageError> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT data FROM buckets")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|(data,)| serde_json::from_str(&data).map_err(Into::into))
            .collect()
    }

    async fn delete_bucket(&self, name: &str) -> Result<(), S3MetadataStorageError> {
        sqlx::query("DELETE FROM buckets WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM bucket_metadata WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn store_bucket_metadata(&self, bucket: &str, metadata: &BucketMetadataBundle) -> Result<(), S3MetadataStorageError> {
        let data = serde_json::to_string(metadata)?;
        sqlx::query("INSERT OR REPLACE INTO bucket_metadata (name, data) VALUES (?, ?)")
            .bind(bucket)
            .bind(&data)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn load_bucket_metadata(&self, bucket: &str) -> Result<Option<BucketMetadataBundle>, S3MetadataStorageError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT data FROM bucket_metadata WHERE name = ?")
            .bind(bucket)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some((data,)) => Ok(Some(serde_json::from_str(&data)?)),
            None => Ok(None),
        }
    }
}
