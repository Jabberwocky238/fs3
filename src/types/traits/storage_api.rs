use async_trait::async_trait;
use crate::types::s3::core::*;
use crate::types::s3::storage_types::*;
use crate::types::s3::object_layer_types::Context;
use crate::types::errors::*;

#[async_trait]
pub trait StorageAPI: Send + Sync {
    // Volume operations
    async fn make_vol(&self, ctx: &Context, volume: &str) -> Result<(), StorageError>;
    async fn list_vols(&self, ctx: &Context) -> Result<Vec<VolInfo>, StorageError>;
    async fn stat_vol(&self, ctx: &Context, volume: &str) -> Result<VolInfo, StorageError>;
    async fn delete_vol(&self, ctx: &Context, volume: &str, force: bool) -> Result<(), StorageError>;

    // Metadata operations
    async fn read_version(&self, ctx: &Context, volume: &str, path: &str, version_id: &str) -> Result<FileInfo, StorageError>;
    async fn write_metadata(&self, ctx: &Context, volume: &str, path: &str, fi: FileInfo) -> Result<(), StorageError>;
    async fn delete_version(&self, ctx: &Context, volume: &str, path: &str, fi: FileInfo) -> Result<(), StorageError>;

    // File operations
    async fn read_file(&self, ctx: &Context, volume: &str, path: &str, offset: i64, buf: &mut [u8]) -> Result<i64, StorageError>;
    async fn create_file(&self, ctx: &Context, volume: &str, path: &str, size: i64, reader: BoxByteStream) -> Result<(), StorageError>;
    async fn append_file(&self, ctx: &Context, volume: &str, path: &str, buf: &[u8]) -> Result<(), StorageError>;
    async fn rename_file(&self, ctx: &Context, src_vol: &str, src_path: &str, dst_vol: &str, dst_path: &str) -> Result<(), StorageError>;
}
