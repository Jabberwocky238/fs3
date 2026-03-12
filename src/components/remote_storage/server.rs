use std::sync::Arc;

use crate::types::FS3Error;
use crate::types::storage_endpoint::StorageEndpoint;
use crate::types::traits::storage_api::StorageAPI;

pub struct RemoteStorageServer {
    endpoint: StorageEndpoint,
    storage: Arc<dyn StorageAPI<FS3Error>>,
}

impl RemoteStorageServer {
    pub fn new(endpoint: StorageEndpoint, storage: Arc<dyn StorageAPI<FS3Error>>) -> Self {
        Self { endpoint, storage }
    }

    pub fn endpoint(&self) -> &StorageEndpoint {
        &self.endpoint
    }

    pub fn storage(&self) -> &Arc<dyn StorageAPI<FS3Error>> {
        &self.storage
    }
}
