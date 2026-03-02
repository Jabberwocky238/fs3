use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

mod bucket;
mod object;
mod multipart;

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
