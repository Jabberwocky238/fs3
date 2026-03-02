use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::types::s3::core::*;
use crate::types::errors::S3MetadataStorageError;

mod bucket;
mod multipart;
mod object;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct JsonMetadataSnapshot {
    pub buckets: Vec<S3Bucket>,
    pub bucket_metadata: Vec<(String, BucketMetadataBundle)>,
    pub objects: Vec<S3Object>,
    pub multiparts: Vec<MultipartUpload>,
    pub multipart_parts: Vec<(String, Vec<UploadedPart>)>,
}

#[derive(Debug, Clone)]
pub struct JsonMetadataStorage {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl JsonMetadataStorage {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            lock: Arc::new(Mutex::new(())),
        }
    }

    fn load_sync(&self) -> Result<JsonMetadataSnapshot, S3MetadataStorageError> {
        if !self.path.exists() {
            return Ok(JsonMetadataSnapshot::default());
        }
        let data = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&data)?)
    }

    fn save_sync(&self, snap: &JsonMetadataSnapshot) -> Result<(), S3MetadataStorageError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(snap)?;
        std::fs::write(&self.path, data)?;
        Ok(())
    }
}
