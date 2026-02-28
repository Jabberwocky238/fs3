use std::path::PathBuf;
use std::sync::Arc;


use crate::policy::PolicyGroup;
use crate::config::{StorageOptions, StorageKind};
use crate::storage::BucketMetaStore;

use crate::storage::PolicyStore;
#[cfg(feature = "multi-user")]
use crate::storage::UserStore;
#[cfg(feature = "storage-k8sconfigmap")]
use crate::storage::impl_configmap::ConfigMapStore;
use crate::storage::impl_json::JsonFileStore;
use crate::storage::impl_memory::InMemoryStore;
#[cfg(feature = "storage-postgres")]
use crate::storage::impl_postgres::PostgresStore;
#[cfg(feature = "storage-sqlite")]
use crate::storage::impl_sqlite::SqliteStore;
#[cfg(feature = "multi-user")]
use crate::storage::types::UserRecord;
use crate::storage::types::{BucketMetadata, StorageError, StorageSnapshot};

pub enum StorageBackend {
    Memory(InMemoryStore),
    Json(JsonFileStore),
    #[cfg(feature = "storage-sqlite")]
    Sqlite(SqliteStore),
    #[cfg(feature = "storage-postgres")]
    Postgres(PostgresStore),
    #[cfg(feature = "storage-k8sconfigmap")]
    ConfigMap(ConfigMapStore),
}

pub async fn new_store(
    cfg: &StorageOptions,
    seed: StorageSnapshot,
) -> Result<Arc<StorageBackend>, StorageError> {
    let backend = match cfg.kind {
        StorageKind::Memory => StorageBackend::Memory(InMemoryStore::new(seed)),
        StorageKind::Json => StorageBackend::Json(JsonFileStore::new(PathBuf::from(&cfg.json_path), seed)?),
        #[cfg(feature = "storage-sqlite")]
        StorageKind::Sqlite => StorageBackend::Sqlite(SqliteStore::new(cfg.dsn.clone(), seed).await?),
        #[cfg(feature = "storage-postgres")]
        StorageKind::Postgres => StorageBackend::Postgres(PostgresStore::new(cfg.dsn.clone(), seed).await?),
        #[cfg(feature = "storage-k8sconfigmap")]
        StorageKind::ConfigMap => {
            StorageBackend::ConfigMap(ConfigMapStore::new(cfg.configmap_name.clone(), seed).await?)
        }
        #[allow(unreachable_patterns)]
        _ => return Err(StorageError::UnsupportedBackend(format!("{:?}", cfg.kind))),
    };
    Ok(Arc::new(backend))
}

#[cfg(feature = "multi-user")]
#[async_trait::async_trait]
impl UserStore for StorageBackend {
    async fn list_users(&self) -> Result<Vec<UserRecord>, StorageError> {
        match self {
            StorageBackend::Memory(s) => s.list_users().await,
            StorageBackend::Json(s) => s.list_users().await,
            #[cfg(feature = "storage-sqlite")]
            StorageBackend::Sqlite(s) => s.list_users().await,
            #[cfg(feature = "storage-postgres")]
            StorageBackend::Postgres(s) => s.list_users().await,
            #[cfg(feature = "storage-k8sconfigmap")]
            StorageBackend::ConfigMap(s) => s.list_users().await,
        }
    }

    async fn save_users(&self, users: Vec<UserRecord>) -> Result<(), StorageError> {
        match self {
            StorageBackend::Memory(s) => s.save_users(users).await,
            StorageBackend::Json(s) => s.save_users(users).await,
            #[cfg(feature = "storage-sqlite")]
            StorageBackend::Sqlite(s) => s.save_users(users).await,
            #[cfg(feature = "storage-postgres")]
            StorageBackend::Postgres(s) => s.save_users(users).await,
            #[cfg(feature = "storage-k8sconfigmap")]
            StorageBackend::ConfigMap(s) => s.save_users(users).await,
        }
    }
}


#[async_trait::async_trait]
impl PolicyStore for StorageBackend {
    async fn list_policy_groups(&self) -> Result<Vec<PolicyGroup>, StorageError> {
        match self {
                StorageBackend::Memory(s) => s.list_policy_groups().await,
            StorageBackend::Json(s) => s.list_policy_groups().await,
            #[cfg(feature = "storage-sqlite")]
            StorageBackend::Sqlite(s) => s.list_policy_groups().await,
            #[cfg(feature = "storage-postgres")]
            StorageBackend::Postgres(s) => s.list_policy_groups().await,
            #[cfg(feature = "storage-k8sconfigmap")]
            StorageBackend::ConfigMap(s) => s.list_policy_groups().await,
        }
    }

    async fn save_policy_groups(&self, groups: Vec<PolicyGroup>) -> Result<(), StorageError> {
        match self {
                StorageBackend::Memory(s) => s.save_policy_groups(groups).await,
            StorageBackend::Json(s) => s.save_policy_groups(groups).await,
            #[cfg(feature = "storage-sqlite")]
            StorageBackend::Sqlite(s) => s.save_policy_groups(groups).await,
            #[cfg(feature = "storage-postgres")]
            StorageBackend::Postgres(s) => s.save_policy_groups(groups).await,
            #[cfg(feature = "storage-k8sconfigmap")]
            StorageBackend::ConfigMap(s) => s.save_policy_groups(groups).await,
        }
    }
}

#[async_trait::async_trait]
impl BucketMetaStore for StorageBackend {
    async fn list_bucket_metadata(&self) -> Result<Vec<BucketMetadata>, StorageError> {
        match self {
            StorageBackend::Memory(s) => s.list_bucket_metadata().await,
            StorageBackend::Json(s) => s.list_bucket_metadata().await,
            #[cfg(feature = "storage-sqlite")]
            StorageBackend::Sqlite(s) => s.list_bucket_metadata().await,
            #[cfg(feature = "storage-postgres")]
            StorageBackend::Postgres(s) => s.list_bucket_metadata().await,
            #[cfg(feature = "storage-k8sconfigmap")]
            StorageBackend::ConfigMap(s) => s.list_bucket_metadata().await,
        }
    }

    async fn upsert_bucket_metadata(&self, meta: BucketMetadata) -> Result<(), StorageError> {
        match self {
            StorageBackend::Memory(s) => s.upsert_bucket_metadata(meta).await,
            StorageBackend::Json(s) => s.upsert_bucket_metadata(meta).await,
            #[cfg(feature = "storage-sqlite")]
            StorageBackend::Sqlite(s) => s.upsert_bucket_metadata(meta).await,
            #[cfg(feature = "storage-postgres")]
            StorageBackend::Postgres(s) => s.upsert_bucket_metadata(meta).await,
            #[cfg(feature = "storage-k8sconfigmap")]
            StorageBackend::ConfigMap(s) => s.upsert_bucket_metadata(meta).await,
        }
    }
}
