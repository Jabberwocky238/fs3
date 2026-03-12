use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::s3::object_layer_types::Context;
use crate::types::s3::storage_types::*;
use crate::types::traits::StdError;

#[async_trait]
pub trait StorageVolume: Send + Sync {
    type Error: StdError;

    async fn make_vol(&self, ctx: &Context, volume: &str) -> Result<(), Self::Error>;
    async fn list_vols(&self, ctx: &Context) -> Result<Vec<VolInfo>, Self::Error>;
    async fn stat_vol(&self, ctx: &Context, volume: &str) -> Result<VolInfo, Self::Error>;
    async fn delete_vol(&self, ctx: &Context, volume: &str, force: bool) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait StorageMetadata: Send + Sync {
    type Error: StdError;

    async fn read_version(&self, ctx: &Context, volume: &str, path: &str, version_id: &str) -> Result<FileInfo, Self::Error>;
    async fn write_all(&self, ctx: &Context, volume: &str, path: &str, data: &[u8], opts: WriteAllOptions) -> Result<(), Self::Error>;
    async fn write_metadata(&self, ctx: &Context, volume: &str, path: &str, fi: FileInfo) -> Result<(), Self::Error>;
    async fn rename_data(&self, ctx: &Context, src_volume: &str, src_path: &str, fi: FileInfo, dst_volume: &str, dst_path: &str, opts: RenameDataOptions) -> Result<RenameDataResult, Self::Error>;
    async fn delete_version(&self, ctx: &Context, volume: &str, path: &str, fi: FileInfo) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait StorageFile: Send + Sync {
    type Error: StdError;

    async fn read_file(&self, ctx: &Context, volume: &str, path: &str, offset: i64, buf: &mut [u8]) -> Result<i64, Self::Error>;
    async fn create_file(&self, ctx: &Context, volume: &str, path: &str, size: i64, reader: BoxByteStream, opts: CreateFileOptions) -> Result<u64, Self::Error>;
    async fn append_file(&self, ctx: &Context, volume: &str, path: &str, buf: &[u8]) -> Result<(), Self::Error>;
    async fn rename_file(&self, ctx: &Context, src_vol: &str, src_path: &str, dst_vol: &str, dst_path: &str) -> Result<(), Self::Error>;
    async fn delete_path(&self, ctx: &Context, volume: &str, path: &str, opts: DeletePathOptions) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait StorageBucketConfig: Send + Sync {
    type Error: StdError;

    async fn read_bucket_policy(&self, ctx: &Context, bucket: &str) -> Result<Option<String>, Self::Error>;
    async fn write_bucket_policy(&self, ctx: &Context, bucket: &str, policy: &str) -> Result<(), Self::Error>;
    async fn delete_bucket_policy(&self, ctx: &Context, bucket: &str) -> Result<(), Self::Error>;

    async fn read_bucket_tags(&self, ctx: &Context, bucket: &str) -> Result<Option<String>, Self::Error>;
    async fn write_bucket_tags(&self, ctx: &Context, bucket: &str, tags: &str) -> Result<(), Self::Error>;
    async fn delete_bucket_tags(&self, ctx: &Context, bucket: &str) -> Result<(), Self::Error>;

    async fn read_bucket_versioning(&self, ctx: &Context, bucket: &str) -> Result<Option<String>, Self::Error>;
    async fn write_bucket_versioning(&self, ctx: &Context, bucket: &str, status: &str) -> Result<(), Self::Error>;

    async fn read_bucket_cors(&self, ctx: &Context, bucket: &str) -> Result<Option<String>, Self::Error>;
    async fn write_bucket_cors(&self, ctx: &Context, bucket: &str, cors: &str) -> Result<(), Self::Error>;
    async fn delete_bucket_cors(&self, ctx: &Context, bucket: &str) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait StorageObjectConfig: Send + Sync {
    type Error: StdError;

    async fn read_object_tags(&self, ctx: &Context, bucket: &str, key: &str) -> Result<Option<String>, Self::Error>;
    async fn write_object_tags(&self, ctx: &Context, bucket: &str, key: &str, tags: &str) -> Result<(), Self::Error>;
    async fn delete_object_tags(&self, ctx: &Context, bucket: &str, key: &str) -> Result<(), Self::Error>;
}

pub trait StorageAPI<E>:
    StorageVolume<Error = E>
    + StorageMetadata<Error = E>
    + StorageFile<Error = E>
    + StorageBucketConfig<Error = E>
    + StorageObjectConfig<Error = E>
where
    E: StdError,
{
}

impl<T, E> StorageAPI<E> for T
where
    T: StorageVolume<Error = E>
        + StorageMetadata<Error = E>
        + StorageFile<Error = E>
        + StorageBucketConfig<Error = E>
        + StorageObjectConfig<Error = E>,
    E: StdError,
{
}
