use async_trait::async_trait;
use futures::StreamExt;

use crate::types::s3::core::*;
use crate::types::errors::S3MountError;
use crate::types::traits::s3_mount::S3MountMultipart;

use super::MemoryMount;

#[async_trait]
impl S3MountMultipart for MemoryMount {
    async fn write_part(&self, bucket: &str, key: &str, upload_id: &str, part_number: u32, body: BoxByteStream) -> Result<u64, S3MountError> {
        let mut buf = Vec::new();
        let mut stream = body;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| S3MountError::Io(e.to_string()))?;
            buf.extend_from_slice(&chunk);
        }
        let size = buf.len() as u64;
        let mut state = self.state.write().await;
        state.parts.insert((bucket.to_owned(), key.to_owned(), upload_id.to_owned(), part_number), buf);
        Ok(size)
    }

    async fn assemble_parts(&self, bucket: &str, key: &str, upload_id: &str, parts: &[UploadedPart]) -> Result<u64, S3MountError> {
        let mut state = self.state.write().await;
        let mut assembled = Vec::new();
        for part in parts {
            let pk = (bucket.to_owned(), key.to_owned(), upload_id.to_owned(), part.part_number);
            let data = state.parts.get(&pk)
                .ok_or_else(|| S3MountError::Io(format!("part not found: {}/{} upload={} part={}", bucket, key, upload_id, part.part_number)))?;
            assembled.extend_from_slice(data);
        }
        let size = assembled.len() as u64;
        let b = state.buckets.get_mut(bucket)
            .ok_or_else(|| S3MountError::NoSuchBucket(bucket.to_owned()))?;
        b.insert(key.to_owned(), assembled);
        Ok(size)
    }

    async fn cleanup_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<(), S3MountError> {
        let mut state = self.state.write().await;
        state.parts.retain(|k, _| !(k.0 == bucket && k.1 == key && k.2 == upload_id));
        Ok(())
    }
}
