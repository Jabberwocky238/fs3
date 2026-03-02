use async_trait::async_trait;

use crate::types::traits::s3_mount::S3MountBucket;

use super::{LocalFsMount, LocalFsMountError};

#[async_trait]
impl S3MountBucket<LocalFsMountError> for LocalFsMount {
    async fn create_bucket_dir(&self, bucket: &str) -> Result<(), LocalFsMountError> {
        tokio::fs::create_dir_all(self.bucket_path(bucket)).await?;
        Ok(())
    }

    async fn delete_bucket_dir(&self, bucket: &str) -> Result<(), LocalFsMountError> {
        let path = self.bucket_path(bucket);
        if path.exists() {
            tokio::fs::remove_dir_all(path).await?;
        }
        Ok(())
    }

    async fn bucket_dir_exists(&self, bucket: &str) -> Result<bool, LocalFsMountError> {
        Ok(self.bucket_path(bucket).is_dir())
    }
}
