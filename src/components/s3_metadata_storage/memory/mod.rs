use crate::types::errors::S3MetadataStorageError;

use super::sqlite::SqliteMetadataStorage;

pub struct MemoryMetadataStorage {}

impl MemoryMetadataStorage {
    pub async fn new() -> Result<SqliteMetadataStorage, S3MetadataStorageError> {
        Ok(SqliteMetadataStorage::new("sqlite::memory:").await?)
    }
}
