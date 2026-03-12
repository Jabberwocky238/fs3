use std::sync::Arc;
use crate::types::FS3Error;
use crate::types::traits::object_layer::ObjectLayer;
use crate::types::traits::storage_api::StorageAPI;

mod bucket;
mod object;
mod multipart;
mod config;
mod website;

pub struct FS3Engine {
    pub object_layer: Arc<dyn ObjectLayer<FS3Error>>,
    pub storage: Arc<dyn StorageAPI<FS3Error>>,
}

impl FS3Engine {
    pub fn new(object_layer: Arc<dyn ObjectLayer<FS3Error>>, storage: Arc<dyn StorageAPI<FS3Error>>) -> Self {
        Self { object_layer, storage }
    }
}

