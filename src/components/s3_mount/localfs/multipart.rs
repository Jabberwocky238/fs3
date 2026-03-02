use async_trait::async_trait;
use futures::StreamExt;

use crate::types::s3::core::*;
use crate::types::traits::s3_mount::S3MountMultipart;

use super::{LocalFsMount, LocalFsMountError};

#[async_trait]
impl S3MountMultipart<LocalFsMountError> for LocalFsMount {
    async fn write_part(&self, bucket: &str, key: &str, upload_id: &str, part_number: u32, body: BoxByteStream) -> Result<u64, LocalFsMountError> {
        let dir = self.parts_dir(bucket, key, upload_id)?;
        tokio::fs::create_dir_all(&dir).await?;
        let part_path = dir.join(format!("{part_number}"));
        let mut buf = Vec::new();
        let mut stream = body;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| LocalFsMountError::Io(e))?;
            buf.extend_from_slice(&chunk);
        }
        let size = buf.len() as u64;
        tokio::fs::write(&part_path, &buf).await?;
        Ok(size)
    }

    async fn assemble_parts(&self, bucket: &str, key: &str, upload_id: &str, parts: &[UploadedPart]) -> Result<u64, LocalFsMountError> {
        let dir = self.parts_dir(bucket, key, upload_id)?;
        let obj_path = self.object_path(bucket, key)?;
        if let Some(parent) = obj_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut assembled = Vec::new();
        for part in parts {
            let part_path = dir.join(format!("{}", part.part_number));
            let data = tokio::fs::read(&part_path).await?;
            assembled.extend_from_slice(&data);
        }
        let size = assembled.len() as u64;
        tokio::fs::write(&obj_path, &assembled).await?;
        Ok(size)
    }

    async fn cleanup_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<(), LocalFsMountError> {
        let dir = self.parts_dir(bucket, key, upload_id)?;
        if dir.exists() {
            tokio::fs::remove_dir_all(&dir).await?;
        }
        Ok(())
    }
}
