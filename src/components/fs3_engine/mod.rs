use std::sync::Arc;
use crate::types::traits::object_layer::ObjectLayer;

mod bucket;
mod config;
mod multipart;
mod object;

pub struct FS3Engine {
    pub object_layer: Arc<dyn ObjectLayer>,
}

impl FS3Engine {
    pub fn new(object_layer: Arc<dyn ObjectLayer>) -> Self {
        Self { object_layer }
    }
}
