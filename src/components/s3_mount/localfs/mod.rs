use std::path::PathBuf;

use crate::types::errors::S3MountError;

mod bucket;
mod multipart;
mod object;

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

    fn object_path(&self, bucket: &str, key: &str) -> Result<PathBuf, S3MountError> {
        let rel = normalize_key(key)?;
        Ok(self.bucket_path(bucket).join(rel))
    }

    fn parts_dir(&self, bucket: &str, key: &str, upload_id: &str) -> Result<PathBuf, S3MountError> {
        let rel = normalize_key(key)?;
        let parts_name = format!(".parts-{upload_id}");
        Ok(self.bucket_path(bucket).join(rel).with_extension(parts_name))
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
