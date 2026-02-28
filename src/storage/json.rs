use std::fs;
use std::path::PathBuf;

use tracing::{debug, info, warn};

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

pub struct JsonFileStore {
    path: PathBuf,
    lock: std::sync::Mutex<()>,
}

impl JsonFileStore {
    pub fn new(path: PathBuf, seed: StorageSnapshot) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| StorageError::Io(e.to_string()))?;
        }
        if !path.exists() {
            info!(path = %path.display(), "creating storage file with seed data");
            let text = serde_json::to_string_pretty(&seed)
                .map_err(|e| StorageError::Serde(e.to_string()))?;
            fs::write(&path, text).map_err(|e| StorageError::Io(e.to_string()))?;
        } else {
            debug!(path = %path.display(), "storage file already exists");
        }
        Ok(Self {
            path,
            lock: std::sync::Mutex::new(()),
        })
    }

    fn load(&self) -> Result<StorageSnapshot, StorageError> {
        let data = fs::read_to_string(&self.path).map_err(|e| {
            warn!(path = %self.path.display(), error = %e, "failed to read storage file");
            StorageError::Io(e.to_string())
        })?;
        serde_json::from_str(&data).map_err(|e| {
            warn!(path = %self.path.display(), error = %e, "failed to parse storage json");
            StorageError::Serde(e.to_string())
        })
    }

    fn save(&self, snapshot: &StorageSnapshot) -> Result<(), StorageError> {
        let text = serde_json::to_string_pretty(snapshot)
            .map_err(|e| StorageError::Serde(e.to_string()))?;
        fs::write(&self.path, text).map_err(|e| {
            warn!(path = %self.path.display(), error = %e, "failed to write storage file");
            StorageError::Io(e.to_string())
        })?;
        debug!(path = %self.path.display(), "storage file saved");
        Ok(())
    }
}

#[cfg(feature = "multi-user")]
#[async_trait::async_trait]
impl UserStore for JsonFileStore {
    async fn list_users(&self) -> Result<Vec<UserRecord>, StorageError> {
        let _g = self
            .lock
            .lock()
            .map_err(|e| StorageError::Io(e.to_string()))?;
        Ok(self.load()?.users)
    }

    async fn save_users(&self, users: Vec<UserRecord>) -> Result<(), StorageError> {
        let _g = self
            .lock
            .lock()
            .map_err(|e| StorageError::Io(e.to_string()))?;
        let mut snap = self.load()?;
        snap.users = users;
        self.save(&snap)
    }
}

#[cfg(feature = "policy")]
#[async_trait::async_trait]
impl PolicyStore for JsonFileStore {
    async fn list_policy_groups(&self) -> Result<Vec<PolicyGroup>, StorageError> {
        let _g = self
            .lock
            .lock()
            .map_err(|e| StorageError::Io(e.to_string()))?;
        Ok(self.load()?.policy_groups)
    }

    async fn save_policy_groups(&self, groups: Vec<PolicyGroup>) -> Result<(), StorageError> {
        let _g = self
            .lock
            .lock()
            .map_err(|e| StorageError::Io(e.to_string()))?;
        let mut snap = self.load()?;
        snap.policy_groups = groups;
        self.save(&snap)
    }
}

#[async_trait::async_trait]
impl BucketMetaStore for JsonFileStore {
    async fn list_bucket_metadata(&self) -> Result<Vec<BucketMetadata>, StorageError> {
        let _g = self
            .lock
            .lock()
            .map_err(|e| StorageError::Io(e.to_string()))?;
        Ok(self.load()?.bucket_metadata)
    }

    async fn upsert_bucket_metadata(&self, meta: BucketMetadata) -> Result<(), StorageError> {
        let _g = self
            .lock
            .lock()
            .map_err(|e| StorageError::Io(e.to_string()))?;
        let mut snap = self.load()?;
        if let Some(item) = snap
            .bucket_metadata
            .iter_mut()
            .find(|x| x.bucket == meta.bucket)
        {
            *item = meta;
        } else {
            snap.bucket_metadata.push(meta);
        }
        self.save(&snap)
    }
}
