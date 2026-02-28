
use crate::policy::PolicyGroup;
#[cfg(feature = "multi-user")]
use crate::storage::types::UserRecord;
use crate::storage::types::{BucketMetadata, StorageError};

#[cfg(feature = "multi-user")]
#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn list_users(&self) -> Result<Vec<UserRecord>, StorageError>;
    async fn save_users(&self, users: Vec<UserRecord>) -> Result<(), StorageError>;
}


#[async_trait::async_trait]
pub trait PolicyStore: Send + Sync {
    async fn list_policy_groups(&self) -> Result<Vec<PolicyGroup>, StorageError>;
    async fn save_policy_groups(&self, groups: Vec<PolicyGroup>) -> Result<(), StorageError>;
}

#[async_trait::async_trait]
pub trait BucketMetaStore: Send + Sync {
    async fn list_bucket_metadata(&self) -> Result<Vec<BucketMetadata>, StorageError>;
    async fn upsert_bucket_metadata(&self, meta: BucketMetadata) -> Result<(), StorageError>;
}

pub mod factory;
pub mod impl_json;
pub mod types;
pub mod impl_memory;

#[cfg(feature = "storage-k8sconfigmap")]
pub mod impl_configmap;
#[cfg(feature = "storage-postgres")]
pub mod impl_postgres;
#[cfg(feature = "storage-sqlite")]
pub mod impl_sqlite;
