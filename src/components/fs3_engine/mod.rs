use std::sync::Arc;
use crate::types::traits::object_layer::ObjectLayer;
use crate::types::traits::storage_api::StorageAPI;

mod bucket;
mod object;
mod multipart;
mod config;

pub struct FS3Engine {
    pub object_layer: Arc<dyn ObjectLayer>,
    pub storage: Arc<dyn StorageAPI>,
}

impl FS3Engine {
    pub fn new(object_layer: Arc<dyn ObjectLayer>, storage: Arc<dyn StorageAPI>) -> Self {
        Self { object_layer, storage }
    }
}
