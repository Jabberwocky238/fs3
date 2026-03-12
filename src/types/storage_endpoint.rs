use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageEndpoint {
    pub node_id: String,
    pub address: String,
    pub storage_id: String,
    pub is_local: bool,
    pub local_path: Option<PathBuf>,
}

impl StorageEndpoint {
    pub fn local(
        node_id: impl Into<String>,
        address: impl Into<String>,
        storage_id: impl Into<String>,
        local_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            address: address.into(),
            storage_id: storage_id.into(),
            is_local: true,
            local_path: Some(local_path.into()),
        }
    }

    pub fn remote(
        node_id: impl Into<String>,
        address: impl Into<String>,
        storage_id: impl Into<String>,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            address: address.into(),
            storage_id: storage_id.into(),
            is_local: false,
            local_path: None,
        }
    }
}
