use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::PolicyGroup;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserRecord {
    pub user_id: String,
    pub enabled: bool,
    pub access_key: String,
    pub secret_key: String,
    pub groups: Vec<String>,
    pub attrs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BucketMetadata {
    pub bucket: String,
    pub owner: String,
    pub labels: HashMap<String, String>,
    pub attrs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSnapshot {
    pub users: Vec<UserRecord>,
    pub policies: Vec<PolicyGroup>,
    pub bucket_metadata: Vec<BucketMetadata>,
}

impl Default for StorageSnapshot {
    fn default() -> Self {
        Self {
            users: Vec::new(),
            policies: Vec::new(),
            bucket_metadata: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("unsupported backend kind: {0}")]
    UnsupportedBackend(String),
    #[error("storage io error: {0}")]
    Io(String),
    #[error("storage serialization error: {0}")]
    Serde(String),
    #[error("storage db error: {0}")]
    Db(String),
}
