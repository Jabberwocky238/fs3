use sqlx::SqlitePool;

use crate::types::errors::S3MetadataStorageError;

mod bucket;
mod multipart;
mod object;

#[derive(Debug, Clone)]
pub struct SqliteMetadataStorage {
    pool: SqlitePool,
}

impl SqliteMetadataStorage {
    pub async fn new(url: &str) -> Result<Self, S3MetadataStorageError> {
        let pool = SqlitePool::connect(url).await?;
        let s = Self { pool };
        s.init_tables().await?;
        Ok(s)
    }

    async fn init_tables(&self) -> Result<(), S3MetadataStorageError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS buckets (
                name TEXT PRIMARY KEY,
                data TEXT NOT NULL
            )"
        ).execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS bucket_metadata (
                name TEXT PRIMARY KEY,
                data TEXT NOT NULL
            )"
        ).execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS objects (
                bucket TEXT NOT NULL,
                key TEXT NOT NULL,
                data TEXT NOT NULL,
                PRIMARY KEY (bucket, key)
            )"
        ).execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS multiparts (
                upload_id TEXT PRIMARY KEY,
                bucket TEXT NOT NULL,
                data TEXT NOT NULL
            )"
        ).execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS multipart_parts (
                upload_id TEXT NOT NULL,
                part_number INTEGER NOT NULL,
                data TEXT NOT NULL,
                PRIMARY KEY (upload_id, part_number)
            )"
        ).execute(&self.pool).await?;

        Ok(())
    }
}
