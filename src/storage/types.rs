use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::PolicyGroup;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserRecord {
    pub user_id: String,
    pub enabled: bool,
    #[serde(default)]
    pub access_key: String,
    #[serde(default)]
    pub secret_key: String,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub attrs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BucketMetadata {
    pub bucket: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub attrs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageSnapshot {
    #[serde(default)]
    pub users: Vec<UserRecord>,
    #[serde(default)]
    pub policy_groups: Vec<PolicyGroup>,
    #[serde(default)]
    pub bucket_metadata: Vec<BucketMetadata>,
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
