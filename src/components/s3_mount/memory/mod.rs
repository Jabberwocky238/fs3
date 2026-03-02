use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::traits::s3_handler::S3HandlerBridgeError;

mod bucket;
mod object;
mod multipart;

#[derive(Debug, thiserror::Error)]
pub enum MemoryMountError {
    #[error("no such bucket: {0}")]
    NoSuchBucket(String),
    #[error("no such key: {bucket}/{key}")]
    NoSuchKey { bucket: String, key: String },
    #[error("invalid key: {0}")]
    BadKey(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("{0}")]
    HandlerBridge(#[from] S3HandlerBridgeError),
}

#[derive(Debug, Clone, Default)]
pub struct MemoryMountState {
    pub buckets: HashMap<String, HashMap<String, Vec<u8>>>,
    pub parts: HashMap<(String, String, String, u32), Vec<u8>>,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryMount {
    pub(crate) state: Arc<RwLock<MemoryMountState>>,
}

impl MemoryMount {
    pub fn new() -> Self {
        Self::default()
    }
}
