use async_trait::async_trait;

use crate::types::s3::mount_error::S3MountError;
use crate::types::traits::s3_mount::S3MountBucket;

use super::LocalFsMount;

#[async_trait]
impl S3MountBucket for LocalFsMount {
    async fn create_bucket_dir(&self, bucket: &str) -> Result<(), S3MountError> {
        tokio::fs::create_dir_all(self.bucket_path(bucket)).await?;
        Ok(())
    }

    async fn delete_bucket_dir(&self, bucket: &str) -> Result<(), S3MountError> {
        let path = self.bucket_path(bucket);
        if path.exists() {
            tokio::fs::remove_dir_all(path).await?;
        }
        Ok(())
    }

    async fn bucket_dir_exists(&self, bucket: &str) -> Result<bool, S3MountError> {
        Ok(self.bucket_path(bucket).is_dir())
    }
}
