use tokio::sync::Mutex;

#[cfg(feature = "policy")]
use crate::config::PolicyGroup;
use crate::storage::BucketMetaStore;
#[cfg(feature = "policy")]
use crate::storage::PolicyStore;
#[cfg(feature = "multi-user")]
use crate::storage::UserStore;
#[cfg(feature = "multi-user")]
use crate::storage::types::UserRecord;
use crate::storage::types::{BucketMetadata, StorageError, StorageSnapshot};

pub struct InMemoryStore {
    snapshot: Mutex<StorageSnapshot>,
}

impl InMemoryStore {
    pub fn new(snapshot: StorageSnapshot) -> Self {
        Self { snapshot: Mutex::new(snapshot) }
    }
}

#[cfg(feature = "multi-user")]
#[async_trait::async_trait]
impl UserStore for InMemoryStore {
    async fn list_users(&self) -> Result<Vec<UserRecord>, StorageError> {
        let snapshot = self.snapshot.lock().await;
        Ok(snapshot.users.clone())
    }

    async fn save_users(&self, users: Vec<UserRecord>) -> Result<(), StorageError> {
        let mut snapshot = self.snapshot.lock().await;
        snapshot.users = users;
        Ok(())
    }
}

#[cfg(feature = "policy")]
#[async_trait::async_trait]
impl PolicyStore for InMemoryStore {
    async fn list_policy_groups(&self) -> Result<Vec<PolicyGroup>, StorageError> {
        let snapshot = self.snapshot.lock().await;
        Ok(snapshot.policies.clone())
    }

    async fn save_policy_groups(&self, groups: Vec<PolicyGroup>) -> Result<(), StorageError> {
        let mut snapshot = self.snapshot.lock().await;
        snapshot.policies = groups;
        Ok(())
    }
}

#[async_trait::async_trait]
impl BucketMetaStore for InMemoryStore {
    async fn list_bucket_metadata(&self) -> Result<Vec<BucketMetadata>, StorageError> {
        let snapshot = self.snapshot.lock().await;
        Ok(snapshot.bucket_metadata.clone())
    }

    async fn upsert_bucket_metadata(&self, meta: BucketMetadata) -> Result<(), StorageError> {
        let mut guard = self.snapshot.lock().await;
        if let Some(existing) = guard
            .bucket_metadata
            .iter_mut()
            .find(|b| b.bucket == meta.bucket)
        {
            *existing = meta;
        } else {
            guard.bucket_metadata.push(meta);
        }
        Ok(())
    }
}
