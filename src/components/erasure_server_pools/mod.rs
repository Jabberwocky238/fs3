use std::sync::Arc;
use crate::types::traits::storage_api::StorageAPI;

mod bucket;
mod object;
mod multipart;

pub struct ErasureServerPools {
    storage: Arc<dyn StorageAPI>,
}

impl ErasureServerPools {
    pub fn new(storage: Arc<dyn StorageAPI>) -> Self {
        Self { storage }
    }

    pub fn storage(&self) -> &Arc<dyn StorageAPI> {
        &self.storage
    }
}
