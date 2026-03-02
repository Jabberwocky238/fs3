use async_trait::async_trait;
use base64::Engine;
use futures::stream;
use futures::StreamExt;
use uuid::Uuid;

use crate::types::errors::S3MountError;
use crate::types::s3::core::*;
use crate::types::traits::s3_mount::{S3MountRead, S3MountWrite};

use super::{LocalFsMount, PartInfo, XlMeta, INLINE_THRESHOLD};

impl LocalFsMount {
    /// Read the full object bytes, handling both inline and part-file storage.
    async fn read_object_bytes(&self, bucket: &str, key: &str) -> Result<Vec<u8>, S3MountError> {
        let meta = self.read_xl_meta(bucket, key).await?;
        if let Some(ref inline) = meta.inline_data {
            base64::engine::general_purpose::STANDARD
                .decode(inline)
                .map_err(|e| S3MountError::Io(e.to_string()))
        } else {
            let mut buf = Vec::new();
            for part in &meta.parts {
                let part_path = self
                    .data_dir_path(bucket, key, &meta.data_dir)?
                    .join(format!("part.{}", part.number));
                let data = tokio::fs::read(&part_path).await.map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        S3MountError::NoSuchKey {
                            bucket: bucket.to_owned(),
                            key: key.to_owned(),
                        }
                    } else {
                        S3MountError::from(e)
                    }
                })?;
                buf.extend_from_slice(&data);
            }
            Ok(buf)
        }
    }
}

#[async_trait]
impl S3MountRead for LocalFsMount {
    async fn read_object(&self, bucket: &str, key: &str) -> Result<BoxByteStream, S3MountError> {
        let data = self.read_object_bytes(bucket, key).await?;
        Ok(Box::pin(stream::once(async move {
            Ok(bytes::Bytes::from(data))
        })))
    }

    async fn read_object_range(
        &self,
        bucket: &str,
        key: &str,
        range: &str,
    ) -> Result<BoxByteStream, S3MountError> {
        let data = self.read_object_bytes(bucket, key).await?;
        let sliced = apply_range(&data, range)?;
        Ok(Box::pin(stream::once(async move { Ok(sliced) })))
    }

    async fn object_exists(&self, bucket: &str, key: &str) -> Result<bool, S3MountError> {
        let path = self.xl_meta_path(bucket, key)?;
        Ok(path.is_file())
    }

    async fn object_size(&self, bucket: &str, key: &str) -> Result<u64, S3MountError> {
        let meta = self.read_xl_meta(bucket, key).await?;
        Ok(meta.size)
    }
}

#[async_trait]
impl S3MountWrite for LocalFsMount {
    async fn write_object(
        &self,
        bucket: &str,
        key: &str,
        body: BoxByteStream,
    ) -> Result<u64, S3MountError> {
        let mut buf = Vec::new();
        let mut stream = body;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| S3MountError::Io(e.to_string()))?;
            buf.extend_from_slice(&chunk);
        }
        let size = buf.len() as u64;
        let data_dir = Uuid::new_v4().to_string();

        let meta = if buf.len() <= INLINE_THRESHOLD {
            // Small object: inline in xl.meta
            XlMeta {
                data_dir,
                parts: vec![PartInfo { number: 1, size }],
                size,
                inline_data: Some(
                    base64::engine::general_purpose::STANDARD.encode(&buf),
                ),
            }
        } else {
            // Large object: write to part file
            let part_dir = self.data_dir_path(bucket, key, &data_dir)?;
            tokio::fs::create_dir_all(&part_dir).await?;
            tokio::fs::write(part_dir.join("part.1"), &buf).await?;
            XlMeta {
                data_dir,
                parts: vec![PartInfo { number: 1, size }],
                size,
                inline_data: None,
            }
        };

        self.write_xl_meta(bucket, key, &meta).await?;
        Ok(size)
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), S3MountError> {
        let dir = self.object_dir(bucket, key)?;
        if dir.exists() {
            tokio::fs::remove_dir_all(&dir).await?;
        }
        Ok(())
    }

    async fn copy_object(
        &self,
        src_bucket: &str,
        src_key: &str,
        dst_bucket: &str,
        dst_key: &str,
    ) -> Result<u64, S3MountError> {
        let data = self.read_object_bytes(src_bucket, src_key).await?;
        let size = data.len() as u64;
        let stream = Box::pin(futures::stream::once(async {
            Ok(bytes::Bytes::from(data)) as Result<bytes::Bytes, std::io::Error>
        }));
        self.write_object(dst_bucket, dst_key, stream).await?;
        Ok(size)
    }
}

fn apply_range(body: &[u8], range: &str) -> Result<bytes::Bytes, S3MountError> {
    let raw = range.trim();
    let raw = raw
        .strip_prefix("bytes=")
        .ok_or_else(|| S3MountError::Io(format!("invalid range: {raw}")))?;
    let (start_s, end_s) = raw
        .split_once('-')
        .ok_or_else(|| S3MountError::Io(format!("invalid range: {range}")))?;
    let len = body.len() as i64;

    let (start, end) = if start_s.is_empty() {
        let suffix: i64 = end_s
            .parse()
            .map_err(|_| S3MountError::Io(format!("invalid range: {range}")))?;
        ((len - suffix).max(0), len.saturating_sub(1))
    } else {
        let start: i64 = start_s
            .parse()
            .map_err(|_| S3MountError::Io(format!("invalid range: {range}")))?;
        let end: i64 = if end_s.is_empty() {
            len.saturating_sub(1)
        } else {
            end_s
                .parse()
                .map_err(|_| S3MountError::Io(format!("invalid range: {range}")))?
        };
        (start, end.min(len.saturating_sub(1)))
    };

    if start < 0 || end < start || start >= len {
        return Err(S3MountError::Io(format!("invalid range: {range}")));
    }
    Ok(bytes::Bytes::copy_from_slice(
        &body[start as usize..=end as usize],
    ))
}
