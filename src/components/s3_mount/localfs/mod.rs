use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::types::errors::S3MountError;

mod bucket;
mod multipart;
mod object;

/// Inline threshold: objects with data <= 128 KiB are stored inline in xl.meta
const INLINE_THRESHOLD: usize = 128 * 1024;

#[derive(Debug, Clone)]
pub struct LocalFsMount {
    root: PathBuf,
}

/// Simplified xl.meta — one JSON file per object, inspired by minio's layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMeta {
    pub data_dir: String,
    pub parts: Vec<PartInfo>,
    pub size: u64,
    /// Inline data (base64-encoded). Present only for small objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartInfo {
    pub number: u32,
    pub size: u64,
}

impl LocalFsMount {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn bucket_path(&self, bucket: &str) -> PathBuf {
        self.root.join(bucket)
    }

    /// Object directory: root/bucket/key/ (a directory, not a file)
    fn object_dir(&self, bucket: &str, key: &str) -> Result<PathBuf, S3MountError> {
        let rel = normalize_key(key)?;
        Ok(self.bucket_path(bucket).join(rel))
    }

    /// xl.meta path: root/bucket/key/xl.meta
    fn xl_meta_path(&self, bucket: &str, key: &str) -> Result<PathBuf, S3MountError> {
        Ok(self.object_dir(bucket, key)?.join("xl.meta"))
    }

    /// Data dir: root/bucket/key/{uuid}/
    fn data_dir_path(&self, bucket: &str, key: &str, data_dir: &str) -> Result<PathBuf, S3MountError> {
        Ok(self.object_dir(bucket, key)?.join(data_dir))
    }

    /// Read and parse xl.meta
    async fn read_xl_meta(&self, bucket: &str, key: &str) -> Result<XlMeta, S3MountError> {
        let path = self.xl_meta_path(bucket, key)?;
        let data = tokio::fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                S3MountError::NoSuchKey {
                    bucket: bucket.to_owned(),
                    key: key.to_owned(),
                }
            } else {
                S3MountError::from(e)
            }
        })?;
        serde_json::from_slice(&data).map_err(|e| S3MountError::Io(e.to_string()))
    }

    /// Write xl.meta to disk
    async fn write_xl_meta(&self, bucket: &str, key: &str, meta: &XlMeta) -> Result<(), S3MountError> {
        let path = self.xl_meta_path(bucket, key)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let data = serde_json::to_vec(meta).map_err(|e| S3MountError::Io(e.to_string()))?;
        tokio::fs::write(&path, &data).await?;
        Ok(())
    }
}

fn normalize_key(key: &str) -> Result<String, S3MountError> {
    let key = key.trim_start_matches('/');
    if key.is_empty() {
        return Err(S3MountError::BadKey("empty key".into()));
    }
    for component in key.split('/') {
        if component == ".." || component == "." {
            return Err(S3MountError::BadKey(format!("path traversal: {key}")));
        }
    }
    Ok(key.to_owned())
}
