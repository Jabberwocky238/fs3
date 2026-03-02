use async_trait::async_trait;
use futures::StreamExt;
use uuid::Uuid;

use crate::types::errors::S3MountError;
use crate::types::s3::core::*;
use crate::types::traits::s3_mount::S3MountMultipart;

use super::{LocalFsMount, PartInfo, XlMeta};

impl LocalFsMount {
    fn parts_staging_dir(&self, bucket: &str, key: &str, upload_id: &str) -> Result<std::path::PathBuf, S3MountError> {
        Ok(self.object_dir(bucket, key)?.join(format!(".parts-{upload_id}")))
    }
}

#[async_trait]
impl S3MountMultipart for LocalFsMount {
    async fn write_part(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        part_number: u32,
        body: BoxByteStream,
    ) -> Result<u64, S3MountError> {
        let dir = self.parts_staging_dir(bucket, key, upload_id)?;
        tokio::fs::create_dir_all(&dir).await?;
        let mut buf = Vec::new();
        let mut stream = body;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| S3MountError::Io(e.to_string()))?;
            buf.extend_from_slice(&chunk);
        }
        let size = buf.len() as u64;
        tokio::fs::write(dir.join(format!("{part_number}")), &buf).await?;
        Ok(size)
    }

    async fn assemble_parts(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        parts: &[UploadedPart],
    ) -> Result<u64, S3MountError> {
        let staging = self.parts_staging_dir(bucket, key, upload_id)?;
        let data_dir = Uuid::new_v4().to_string();
        let dest = self.data_dir_path(bucket, key, &data_dir)?;
        tokio::fs::create_dir_all(&dest).await?;

        let mut total_size = 0u64;
        let mut part_infos = Vec::new();

        for part in parts {
            let src = staging.join(format!("{}", part.part_number));
            let dst = dest.join(format!("part.{}", part.part_number));
            tokio::fs::rename(&src, &dst).await?;
            let size = tokio::fs::metadata(&dst).await?.len();
            total_size += size;
            part_infos.push(PartInfo {
                number: part.part_number,
                size,
            });
        }

        let meta = XlMeta {
            data_dir,
            parts: part_infos,
            size: total_size,
            inline_data: None,
        };
        self.write_xl_meta(bucket, key, &meta).await?;
        Ok(total_size)
    }

    async fn cleanup_parts(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> Result<(), S3MountError> {
        let dir = self.parts_staging_dir(bucket, key, upload_id)?;
        if dir.exists() {
            tokio::fs::remove_dir_all(&dir).await?;
        }
        Ok(())
    }
}
