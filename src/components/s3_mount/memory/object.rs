use async_trait::async_trait;
use futures::stream;
use futures::StreamExt;

use crate::types::s3::core::*;
use crate::types::errors::S3MountError;
use crate::types::traits::s3_mount::{S3MountRead, S3MountWrite};

use super::MemoryMount;

#[async_trait]
impl S3MountRead for MemoryMount {
    async fn read_object(&self, bucket: &str, key: &str) -> Result<BoxByteStream, S3MountError> {
        let state = self.state.read().await;
        let b = state.buckets.get(bucket)
            .ok_or_else(|| S3MountError::NoSuchBucket(bucket.to_owned()))?;
        let data = b.get(key)
            .ok_or_else(|| S3MountError::NoSuchKey { bucket: bucket.to_owned(), key: key.to_owned() })?
            .clone();
        Ok(Box::pin(stream::once(async move { Ok(bytes::Bytes::from(data)) })))
    }

    async fn read_object_range(&self, bucket: &str, key: &str, range: &str) -> Result<BoxByteStream, S3MountError> {
        let state = self.state.read().await;
        let b = state.buckets.get(bucket)
            .ok_or_else(|| S3MountError::NoSuchBucket(bucket.to_owned()))?;
        let data = b.get(key)
            .ok_or_else(|| S3MountError::NoSuchKey { bucket: bucket.to_owned(), key: key.to_owned() })?;
        let sliced = apply_range(data, range)?;
        Ok(Box::pin(stream::once(async move { Ok(sliced) })))
    }

    async fn object_exists(&self, bucket: &str, key: &str) -> Result<bool, S3MountError> {
        let state = self.state.read().await;
        Ok(state.buckets.get(bucket).map_or(false, |b| b.contains_key(key)))
    }

    async fn object_size(&self, bucket: &str, key: &str) -> Result<u64, S3MountError> {
        let state = self.state.read().await;
        let b = state.buckets.get(bucket)
            .ok_or_else(|| S3MountError::NoSuchBucket(bucket.to_owned()))?;
        let data = b.get(key)
            .ok_or_else(|| S3MountError::NoSuchKey { bucket: bucket.to_owned(), key: key.to_owned() })?;
        Ok(data.len() as u64)
    }
}

#[async_trait]
impl S3MountWrite for MemoryMount {
    async fn write_object(&self, bucket: &str, key: &str, body: BoxByteStream) -> Result<u64, S3MountError> {
        let mut buf = Vec::new();
        let mut stream = body;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| S3MountError::Io(e.to_string()))?;
            buf.extend_from_slice(&chunk);
        }
        let size = buf.len() as u64;
        let mut state = self.state.write().await;
        let b = state.buckets.get_mut(bucket)
            .ok_or_else(|| S3MountError::NoSuchBucket(bucket.to_owned()))?;
        b.insert(key.to_owned(), buf);
        Ok(size)
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), S3MountError> {
        let mut state = self.state.write().await;
        let b = state.buckets.get_mut(bucket)
            .ok_or_else(|| S3MountError::NoSuchBucket(bucket.to_owned()))?;
        b.remove(key);
        Ok(())
    }

    async fn copy_object(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str) -> Result<u64, S3MountError> {
        let state = self.state.read().await;
        let sb = state.buckets.get(src_bucket)
            .ok_or_else(|| S3MountError::NoSuchBucket(src_bucket.to_owned()))?;
        let data = sb.get(src_key)
            .ok_or_else(|| S3MountError::NoSuchKey { bucket: src_bucket.to_owned(), key: src_key.to_owned() })?
            .clone();
        drop(state);

        let size = data.len() as u64;
        let mut state = self.state.write().await;
        let db = state.buckets.get_mut(dst_bucket)
            .ok_or_else(|| S3MountError::NoSuchBucket(dst_bucket.to_owned()))?;
        db.insert(dst_key.to_owned(), data);
        Ok(size)
    }
}

fn apply_range(body: &[u8], range: &str) -> Result<bytes::Bytes, S3MountError> {
    let raw = range.trim();
    let raw = raw.strip_prefix("bytes=")
        .ok_or_else(|| S3MountError::Io(format!("invalid range: {raw}")))?;
    let (start_s, end_s) = raw.split_once('-')
        .ok_or_else(|| S3MountError::Io(format!("invalid range: {range}")))?;
    let len = body.len() as i64;

    let (start, end) = if start_s.is_empty() {
        let suffix: i64 = end_s.parse().map_err(|_| S3MountError::Io(format!("invalid range: {range}")))?;
        ((len - suffix).max(0), len.saturating_sub(1))
    } else {
        let start: i64 = start_s.parse().map_err(|_| S3MountError::Io(format!("invalid range: {range}")))?;
        let end: i64 = if end_s.is_empty() { len.saturating_sub(1) } else {
            end_s.parse().map_err(|_| S3MountError::Io(format!("invalid range: {range}")))?
        };
        (start, end.min(len.saturating_sub(1)))
    };

    if start < 0 || end < start || start >= len {
        return Err(S3MountError::Io(format!("invalid range: {range}")));
    }
    Ok(bytes::Bytes::copy_from_slice(&body[start as usize..=end as usize]))
}