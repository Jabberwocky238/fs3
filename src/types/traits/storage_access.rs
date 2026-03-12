use crate::types::traits::StdError;
use crate::types::traits::storage_api::{
    StorageBucketConfig, StorageFile, StorageMetadata, StorageObjectConfig, StorageVolume,
};

pub trait StorageDataAPI<E>:
    StorageVolume<E> + StorageMetadata<E> + StorageFile<E> + Send + Sync
where
    E: StdError,
{
}

impl<T, E> StorageDataAPI<E> for T
where
    T: StorageVolume<E> + StorageMetadata<E> + StorageFile<E> + Send + Sync,
    E: StdError,
{
}

pub trait StorageConfigAPI<E>:
    StorageBucketConfig<E> + StorageObjectConfig<E> + Send + Sync
where
    E: StdError,
{
}

impl<T, E> StorageConfigAPI<E> for T
where
    T: StorageBucketConfig<E> + StorageObjectConfig<E> + Send + Sync,
    E: StdError,
{
}
