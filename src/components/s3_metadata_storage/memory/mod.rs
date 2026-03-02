use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::s3::core::*;

mod bucket;
mod object;
mod multipart;

#[derive(Debug, Clone, Default)]
pub struct MemoryMetadataStorageState {
    pub buckets: HashMap<String, S3Bucket>,
    pub bucket_metadata: HashMap<String, BucketMetadataBundle>,
    pub objects: HashMap<(String, String), S3Object>,
    pub multiparts: HashMap<String, MultipartUpload>,
    pub multipart_parts: HashMap<String, Vec<UploadedPart>>,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryMetadataStorage {
    pub(crate) state: Arc<RwLock<MemoryMetadataStorageState>>,
}

impl MemoryMetadataStorage {
    pub fn new() -> Self {
        Self::default()
    }
}
