use std::path::PathBuf;

use crate::types::traits::s3_handler::S3HandlerBridgeError;

mod bucket;
mod multipart;
mod object;

#[derive(Debug, thiserror::Error)]
pub enum LocalFsMountError {
    #[error("no such bucket: {0}")]
    NoSuchBucket(String),
    #[error("no such key: {bucket}/{key}")]
    NoSuchKey { bucket: String, key: String },
    #[error("invalid key: {0}")]
    BadKey(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    HandlerBridge(#[from] S3HandlerBridgeError),
}

#[derive(Debug, Clone)]
pub struct LocalFsMount {
    root: PathBuf,
}

impl LocalFsMount {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn bucket_path(&self, bucket: &str) -> PathBuf {
        self.root.join(bucket)
    }

    fn object_path(&self, bucket: &str, key: &str) -> Result<PathBuf, LocalFsMountError> {
        let rel = normalize_key(key)?;
        Ok(self.bucket_path(bucket).join(rel))
    }

    fn parts_dir(&self, bucket: &str, key: &str, upload_id: &str) -> Result<PathBuf, LocalFsMountError> {
        let rel = normalize_key(key)?;
        let parts_name = format!(".parts-{upload_id}");
        Ok(self.bucket_path(bucket).join(rel).with_extension(parts_name))
    }
}

fn normalize_key(key: &str) -> Result<String, LocalFsMountError> {
    let key = key.trim_start_matches('/');
    if key.is_empty() {
        return Err(LocalFsMountError::BadKey("empty key".into()));
    }
    for component in key.split('/') {
        if component == ".." || component == "." {
            return Err(LocalFsMountError::BadKey(format!("path traversal: {key}")));
        }
    }
    Ok(key.to_owned())
}
