use std::sync::Arc;
use crate::types::FS3Error;
use crate::types::traits::storage_api::StorageAPI;

mod bucket;
mod object;
mod multipart;
mod write_path;

pub struct ErasureServerPools {
    storage: Arc<dyn StorageAPI<FS3Error>>,
}

impl ErasureServerPools {
    pub fn new(storage: Arc<dyn StorageAPI<FS3Error>>) -> Self {
        Self { storage }
    }

    pub fn storage(&self) -> &Arc<dyn StorageAPI<FS3Error>> {
        &self.storage
    }
}
