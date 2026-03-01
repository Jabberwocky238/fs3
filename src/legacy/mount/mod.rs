pub mod localfs;
pub mod memory;

use std::io;
use std::io::Read;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::config::{MountMode, MountOptions};

#[derive(Debug, Error)]
pub enum MountError {
    #[error("no such bucket")]
    NoSuchBucket,
    #[error("no such key")]
    NoSuchKey,
    #[error("bucket is read-only")]
    ReadOnly,
    #[error("invalid object key")]
    BadKey,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("config error: {0}")]
    Config(String),
}

#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub key: String,
    pub size: i64,
    pub last_modified: DateTime<Utc>,
    pub physical_path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct ListResult {
    pub keys: Vec<ObjectInfo>,
    pub common_prefixes: Vec<String>,
    pub next_token: String,
    pub truncated: bool,
}

pub trait MountManager: Send + Sync {
    fn buckets(&self) -> Vec<String>;
    fn has_bucket(&self, bucket: &str) -> bool;
    fn ensure_bucket(&self, bucket: &str) -> Result<(), MountError>;
    fn open(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(Box<dyn Read + Send>, ObjectInfo), MountError>;
    fn stat(&self, bucket: &str, key: &str) -> Result<ObjectInfo, MountError>;
    fn put(&self, bucket: &str, key: &str, data: &[u8]) -> Result<ObjectInfo, MountError>;
    fn put_reader(
        &self,
        bucket: &str,
        key: &str,
        r: &mut dyn Read,
    ) -> Result<ObjectInfo, MountError>;
    fn delete(&self, bucket: &str, key: &str) -> Result<(), MountError>;
    fn list(
        &self,
        bucket: &str,
        prefix: &str,
        delimiter: &str,
        token: &str,
        max_keys: usize,
    ) -> Result<ListResult, MountError>;
}

pub fn new_mount(opts: &MountOptions) -> Result<Box<dyn MountManager>, MountError> {
    match opts.mode {
        MountMode::Filesystem => Ok(Box::new(localfs::LocalFsMountManager::new(&opts.path)?)),
        MountMode::Memory => Ok(Box::new(memory::MemoryMountManager::new())),
    }
}
