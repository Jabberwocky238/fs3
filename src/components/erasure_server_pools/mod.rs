use crate::types::FS3Error;
use crate::types::traits::storage_api::StorageAPI;
use std::sync::Arc;

mod bucket;
mod multipart;
mod object;
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
