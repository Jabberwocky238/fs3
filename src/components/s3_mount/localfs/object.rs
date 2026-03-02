use async_trait::async_trait;
use futures::stream;
use futures::StreamExt;

use crate::types::s3::core::*;
use crate::types::traits::s3_mount::{S3MountRead, S3MountWrite};

use super::{LocalFsMount, LocalFsMountError};

#[async_trait]
impl S3MountRead<LocalFsMountError> for LocalFsMount {
    async fn read_object(&self, bucket: &str, key: &str) -> Result<BoxByteStream, LocalFsMountError> {
        let path = self.object_path(bucket, key)?;
        let data = tokio::fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                LocalFsMountError::NoSuchKey { bucket: bucket.to_owned(), key: key.to_owned() }
            } else {
                LocalFsMountError::Io(e)
            }
        })?;
        Ok(Box::pin(stream::once(async move { Ok(bytes::Bytes::from(data)) })))
    }

    async fn read_object_range(&self, bucket: &str, key: &str, range: &str) -> Result<BoxByteStream, LocalFsMountError> {
        let path = self.object_path(bucket, key)?;
        let data = tokio::fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                LocalFsMountError::NoSuchKey { bucket: bucket.to_owned(), key: key.to_owned() }
            } else {
                LocalFsMountError::Io(e)
            }
        })?;
        let sliced = apply_range(&data, range)?;
        Ok(Box::pin(stream::once(async move { Ok(sliced) })))
    }

    async fn object_exists(&self, bucket: &str, key: &str) -> Result<bool, LocalFsMountError> {
        let path = self.object_path(bucket, key)?;
        Ok(path.is_file())
    }

    async fn object_size(&self, bucket: &str, key: &str) -> Result<u64, LocalFsMountError> {
        let path = self.object_path(bucket, key)?;
        let meta = tokio::fs::metadata(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                LocalFsMountError::NoSuchKey { bucket: bucket.to_owned(), key: key.to_owned() }
            } else {
                LocalFsMountError::Io(e)
            }
        })?;
        Ok(meta.len())
    }
}

#[async_trait]
impl S3MountWrite<LocalFsMountError> for LocalFsMount {
    async fn write_object(&self, bucket: &str, key: &str, body: BoxByteStream) -> Result<u64, LocalFsMountError> {
        let path = self.object_path(bucket, key)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut buf = Vec::new();
        let mut stream = body;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| LocalFsMountError::Io(e))?;
            buf.extend_from_slice(&chunk);
        }
        let size = buf.len() as u64;
        tokio::fs::write(&path, &buf).await?;
        Ok(size)
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), LocalFsMountError> {
        let path = self.object_path(bucket, key)?;
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }
        Ok(())
    }

    async fn copy_object(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str) -> Result<u64, LocalFsMountError> {
        let src = self.object_path(src_bucket, src_key)?;
        let dst = self.object_path(dst_bucket, dst_key)?;
        if let Some(parent) = dst.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let size = tokio::fs::copy(&src, &dst).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                LocalFsMountError::NoSuchKey { bucket: src_bucket.to_owned(), key: src_key.to_owned() }
            } else {
                LocalFsMountError::Io(e)
            }
        })?;
        Ok(size)
    }
}

fn apply_range(body: &[u8], range: &str) -> Result<bytes::Bytes, LocalFsMountError> {
    let raw = range.trim();
    let raw = raw.strip_prefix("bytes=")
        .ok_or_else(|| LocalFsMountError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("invalid range: {raw}"))))?;
    let (start_s, end_s) = raw.split_once('-')
        .ok_or_else(|| LocalFsMountError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("invalid range: {range}"))))?;
    let len = body.len() as i64;

    let (start, end) = if start_s.is_empty() {
        let suffix: i64 = end_s.parse().map_err(|_| LocalFsMountError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("invalid range: {range}"))))?;
        ((len - suffix).max(0), len.saturating_sub(1))
    } else {
        let start: i64 = start_s.parse().map_err(|_| LocalFsMountError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("invalid range: {range}"))))?;
        let end: i64 = if end_s.is_empty() { len.saturating_sub(1) } else {
            end_s.parse().map_err(|_| LocalFsMountError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("invalid range: {range}"))))?
        };
        (start, end.min(len.saturating_sub(1)))
    };

    if start < 0 || end < start || start >= len {
        return Err(LocalFsMountError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("invalid range: {range}"))));
    }
    Ok(bytes::Bytes::copy_from_slice(&body[start as usize..=end as usize]))
}
