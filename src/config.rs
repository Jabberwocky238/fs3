use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub listen_inner: String,
    pub listen_outer: String,
    pub user_enabled: bool,
    pub multi_bucket_enabled: bool,
    pub mount: MountOptions,
    pub storage: StorageOptions,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_inner: "localhost:3000".to_string(),
            listen_outer: "localhost:3001".to_string(),
            user_enabled: false,
            multi_bucket_enabled: false,
            mount: MountOptions::default(),
            storage: StorageOptions::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MountOptions {
    pub mode: MountMode,
    pub path: String,
}

impl Default for MountOptions {
    fn default() -> Self {
        Self {
            mode: MountMode::Filesystem,
            path: "./".into(),
        }
    }
}

impl MountOptions {
    pub fn memory() -> Self {
        Self {
            mode: MountMode::Memory,
            path: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum MountMode {
    #[default]
    Memory,
    Filesystem,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct PolicyGroup {
    pub name: String,
    pub enabled: bool,
    pub users: Vec<String>,
    pub rules: Vec<PolicyRule>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct PolicyRule {
    pub bucket: String,
    pub prefix: String,
    pub allow: bool,
    pub users: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct StorageOptions {
    pub kind: StorageKind,
    pub json_path: String,
    pub dsn: String,
    pub configmap_name: String,
}

impl Default for StorageOptions {
    fn default() -> Self {
        Self {
            kind: StorageKind::Json,
            json_path: "./storage.json".to_string(),
            dsn: String::new(),
            configmap_name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StorageKind {
    #[default]
    Memory,
    Json,
    Sqlite,
    Postgres,
    ConfigMap,
}
