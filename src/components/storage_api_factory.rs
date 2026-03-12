use std::sync::Arc;

use crate::components::remote_storage::RemoteStorageClient;
use crate::components::xl_storage::XlStorage;
use crate::types::FS3Error;
use crate::types::storage_endpoint::StorageEndpoint;
use crate::types::traits::storage_api::StorageAPI;

pub fn new_storage_api(
    endpoint: &StorageEndpoint,
) -> Result<Arc<dyn StorageAPI<FS3Error>>, FS3Error> {
    if endpoint.is_local {
        let path = endpoint
            .local_path
            .clone()
            .ok_or_else(|| FS3Error::internal("local storage endpoint requires local_path"))?;
        return Ok(Arc::new(XlStorage::new(path)));
    }

    Ok(Arc::new(RemoteStorageClient::new(endpoint.clone())))
}
