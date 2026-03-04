use async_trait::async_trait;
use crate::types::s3::core::*;
use crate::types::s3::storage_types::*;
use crate::types::s3::object_layer_types::Context;
use crate::types::errors::*;

#[async_trait]
pub trait StorageVolume: Send + Sync {
    async fn make_vol(&self, ctx: &Context, volume: &str) -> Result<(), StorageError>;
    async fn list_vols(&self, ctx: &Context) -> Result<Vec<VolInfo>, StorageError>;
    async fn stat_vol(&self, ctx: &Context, volume: &str) -> Result<VolInfo, StorageError>;
    async fn delete_vol(&self, ctx: &Context, volume: &str, force: bool) -> Result<(), StorageError>;
}

#[async_trait]
pub trait StorageMetadata: Send + Sync {
    async fn read_version(&self, ctx: &Context, volume: &str, path: &str, version_id: &str) -> Result<FileInfo, StorageError>;
    async fn write_metadata(&self, ctx: &Context, volume: &str, path: &str, fi: FileInfo) -> Result<(), StorageError>;
    async fn delete_version(&self, ctx: &Context, volume: &str, path: &str, fi: FileInfo) -> Result<(), StorageError>;
}

#[async_trait]
pub trait StorageFile: Send + Sync {
    async fn read_file(&self, ctx: &Context, volume: &str, path: &str, offset: i64, buf: &mut [u8]) -> Result<i64, StorageError>;
    async fn create_file(&self, ctx: &Context, volume: &str, path: &str, size: i64, reader: BoxByteStream) -> Result<(), StorageError>;
    async fn append_file(&self, ctx: &Context, volume: &str, path: &str, buf: &[u8]) -> Result<(), StorageError>;
    async fn rename_file(&self, ctx: &Context, src_vol: &str, src_path: &str, dst_vol: &str, dst_path: &str) -> Result<(), StorageError>;
}

#[async_trait]
pub trait StorageBucketConfig: Send + Sync {
    async fn read_bucket_policy(&self, ctx: &Context, bucket: &str) -> Result<Option<String>, StorageError>;
    async fn write_bucket_policy(&self, ctx: &Context, bucket: &str, policy: &str) -> Result<(), StorageError>;
    async fn delete_bucket_policy(&self, ctx: &Context, bucket: &str) -> Result<(), StorageError>;

    async fn read_bucket_tags(&self, ctx: &Context, bucket: &str) -> Result<Option<String>, StorageError>;
    async fn write_bucket_tags(&self, ctx: &Context, bucket: &str, tags: &str) -> Result<(), StorageError>;
    async fn delete_bucket_tags(&self, ctx: &Context, bucket: &str) -> Result<(), StorageError>;

    async fn read_bucket_versioning(&self, ctx: &Context, bucket: &str) -> Result<Option<String>, StorageError>;
    async fn write_bucket_versioning(&self, ctx: &Context, bucket: &str, status: &str) -> Result<(), StorageError>;
}

pub trait StorageAPI: StorageVolume + StorageMetadata + StorageFile + StorageBucketConfig {}

impl<T> StorageAPI for T where T: StorageVolume + StorageMetadata + StorageFile + StorageBucketConfig {}
