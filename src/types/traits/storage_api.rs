use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::s3::object_layer_types::Context;
use crate::types::s3::storage_types::*;
use crate::types::traits::StdError;

#[async_trait]
pub trait StorageVolume<E>: Send + Sync
where
    E: StdError,
{
    async fn make_vol(&self, ctx: &Context, volume: &str) -> Result<(), E>;
    async fn list_vols(&self, ctx: &Context) -> Result<Vec<VolInfo>, E>;
    async fn stat_vol(&self, ctx: &Context, volume: &str) -> Result<VolInfo, E>;
    async fn delete_vol(&self, ctx: &Context, volume: &str, force: bool) -> Result<(), E>;
}

#[async_trait]
pub trait StorageMetadata<E>: Send + Sync
where
    E: StdError,
{
    async fn read_version(
        &self,
        ctx: &Context,
        volume: &str,
        path: &str,
        version_id: &str,
    ) -> Result<FileInfo, E>;
    async fn write_all(
        &self,
        ctx: &Context,
        volume: &str,
        path: &str,
        data: &[u8],
        opts: WriteAllOptions,
    ) -> Result<(), E>;
    async fn write_metadata(
        &self,
        ctx: &Context,
        volume: &str,
        path: &str,
        fi: FileInfo,
    ) -> Result<(), E>;
    async fn rename_data(
        &self,
        ctx: &Context,
        src_volume: &str,
        src_path: &str,
        fi: FileInfo,
        dst_volume: &str,
        dst_path: &str,
        opts: RenameDataOptions,
    ) -> Result<RenameDataResult, E>;
    async fn delete_version(
        &self,
        ctx: &Context,
        volume: &str,
        path: &str,
        fi: FileInfo,
    ) -> Result<(), E>;
}

#[async_trait]
pub trait StorageFile<E>: Send + Sync
where
    E: StdError,
{
    async fn read_file(
        &self,
        ctx: &Context,
        volume: &str,
        path: &str,
        offset: i64,
        buf: &mut [u8],
    ) -> Result<i64, E>;
    async fn create_file(
        &self,
        ctx: &Context,
        volume: &str,
        path: &str,
        size: i64,
        reader: BoxByteStream,
        opts: CreateFileOptions,
    ) -> Result<u64, E>;
    async fn append_file(
        &self,
        ctx: &Context,
        volume: &str,
        path: &str,
        buf: &[u8],
    ) -> Result<(), E>;
    async fn rename_file(
        &self,
        ctx: &Context,
        src_vol: &str,
        src_path: &str,
        dst_vol: &str,
        dst_path: &str,
    ) -> Result<(), E>;
    async fn delete_path(
        &self,
        ctx: &Context,
        volume: &str,
        path: &str,
        opts: DeletePathOptions,
    ) -> Result<(), E>;
}

#[async_trait]
pub trait StorageBucketConfig<E>: Send + Sync
where
    E: StdError,
{
    async fn read_bucket_policy(&self, ctx: &Context, bucket: &str) -> Result<Option<String>, E>;
    async fn write_bucket_policy(&self, ctx: &Context, bucket: &str, policy: &str)
    -> Result<(), E>;
    async fn delete_bucket_policy(&self, ctx: &Context, bucket: &str) -> Result<(), E>;

    async fn read_bucket_tags(&self, ctx: &Context, bucket: &str) -> Result<Option<String>, E>;
    async fn write_bucket_tags(&self, ctx: &Context, bucket: &str, tags: &str) -> Result<(), E>;
    async fn delete_bucket_tags(&self, ctx: &Context, bucket: &str) -> Result<(), E>;

    async fn read_bucket_versioning(
        &self,
        ctx: &Context,
        bucket: &str,
    ) -> Result<Option<String>, E>;
    async fn write_bucket_versioning(
        &self,
        ctx: &Context,
        bucket: &str,
        status: &str,
    ) -> Result<(), E>;

    async fn read_bucket_cors(&self, ctx: &Context, bucket: &str) -> Result<Option<String>, E>;
    async fn write_bucket_cors(&self, ctx: &Context, bucket: &str, cors: &str) -> Result<(), E>;
    async fn delete_bucket_cors(&self, ctx: &Context, bucket: &str) -> Result<(), E>;
}

#[async_trait]
pub trait StorageObjectConfig<E>: Send + Sync
where
    E: StdError,
{
    async fn read_object_tags(
        &self,
        ctx: &Context,
        bucket: &str,
        key: &str,
    ) -> Result<Option<String>, E>;
    async fn write_object_tags(
        &self,
        ctx: &Context,
        bucket: &str,
        key: &str,
        tags: &str,
    ) -> Result<(), E>;
    async fn delete_object_tags(&self, ctx: &Context, bucket: &str, key: &str) -> Result<(), E>;
}

pub trait StorageAPI<E>:
    StorageVolume<E>
    + StorageMetadata<E>
    + StorageFile<E>
    + StorageBucketConfig<E>
    + StorageObjectConfig<E>
    + Send
    + Sync
where
    E: StdError,
{
}

impl<T, E> StorageAPI<E> for T
where
    T: StorageVolume<E>
        + StorageMetadata<E>
        + StorageFile<E>
        + StorageBucketConfig<E>
        + StorageObjectConfig<E>
        + Send
        + Sync,
    E: StdError,
{
}
