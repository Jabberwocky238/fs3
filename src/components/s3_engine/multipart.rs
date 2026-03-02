use async_trait::async_trait;
use futures::TryStreamExt;

use crate::types::s3::core::*;
use crate::types::errors::S3EngineError;
use crate::types::traits::s3_engine::S3MultipartEngine;
use crate::types::traits::s3_metadata_storage::*;
use crate::types::traits::s3_mount::*;

use super::S3EngineImpl;

#[async_trait]
impl<S, M> S3MultipartEngine for S3EngineImpl<S, M>
where
    S: S3MetadataStorageBucket
        + S3MetadataStorageObject
        + S3MetadataStorageMultipart
        + Send
        + Sync,
    M: S3MountRead
        + S3MountWrite
        + S3MountMultipart
        + Send
        + Sync,
{
    async fn new_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        options: ObjectWriteOptions,
    ) -> Result<MultipartUpload, S3EngineError> {
        self.metadata
            .load_bucket(bucket)
            .await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        let upload = MultipartUpload {
            bucket: bucket.to_owned(),
            key: key.to_owned(),
            upload_id: uuid::Uuid::new_v4().to_string(),
            initiated_at: chrono::Utc::now(),
            storage_class: options.storage_class.clone(),
            user_metadata: options.user_metadata.clone(),
            user_tags: options.user_tags.clone(),
        };
        self.metadata.store_multipart(&upload).await?;
        Ok(upload)
    }

    async fn put_object_part(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        part_number: u32,
        body: BoxByteStream,
    ) -> Result<UploadedPart, S3EngineError> {
        self.metadata
            .load_multipart(upload_id)
            .await?
            .ok_or_else(|| S3EngineError::MultipartNotFound(upload_id.to_owned()))?;
        let chunks: Vec<bytes::Bytes> = body
            .try_collect()
            .await
            .map_err(|e| S3EngineError::Storage(format!("stream error: {e}")))?;
        let mut buf = Vec::new();
        for chunk in &chunks {
            buf.extend_from_slice(chunk);
        }
        let etag = Self::compute_etag(&buf);
        let size = buf.len() as u64;
        let stream: BoxByteStream =
            Box::pin(futures::stream::once(async { Ok(bytes::Bytes::from(buf)) }));
        self.mount
            .write_part(bucket, key, upload_id, part_number, stream)
            .await?;
        let part = UploadedPart {
            part_number,
            etag,
            size,
        };
        self.metadata.store_uploaded_part(upload_id, &part).await?;
        Ok(part)
    }

    async fn copy_object_part(
        &self,
        src_bucket: &str,
        src_key: &str,
        _dst_bucket: &str,
        _dst_key: &str,
        upload_id: &str,
        part_number: u32,
    ) -> Result<UploadedPart, S3EngineError> {
        let upload = self
            .metadata
            .load_multipart(upload_id)
            .await?
            .ok_or_else(|| S3EngineError::MultipartNotFound(upload_id.to_owned()))?;
        let data_stream = self.mount.read_object(src_bucket, src_key).await?;
        let chunks: Vec<bytes::Bytes> = data_stream
            .try_collect()
            .await
            .map_err(|e| S3EngineError::Storage(format!("stream error: {e}")))?;
        let mut buf = Vec::new();
        for chunk in &chunks {
            buf.extend_from_slice(chunk);
        }
        let etag = Self::compute_etag(&buf);
        let size = buf.len() as u64;
        let stream: BoxByteStream =
            Box::pin(futures::stream::once(async { Ok(bytes::Bytes::from(buf)) }));
        self.mount
            .write_part(
                &upload.bucket,
                &upload.key,
                upload_id,
                part_number,
                stream,
            )
            .await?;
        let part = UploadedPart {
            part_number,
            etag,
            size,
        };
        self.metadata.store_uploaded_part(upload_id, &part).await?;
        Ok(part)
    }

    async fn list_object_parts(
        &self,
        _bucket: &str,
        _key: &str,
        upload_id: &str,
    ) -> Result<Vec<UploadedPart>, S3EngineError> {
        self.metadata
            .load_multipart(upload_id)
            .await?
            .ok_or_else(|| S3EngineError::MultipartNotFound(upload_id.to_owned()))?;
        Ok(self.metadata.list_uploaded_parts(upload_id).await?)
    }

    async fn complete_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        completed: CompleteMultipartInput,
    ) -> Result<S3Object, S3EngineError> {
        let upload = self
            .metadata
            .load_multipart(upload_id)
            .await?
            .ok_or_else(|| S3EngineError::MultipartNotFound(upload_id.to_owned()))?;
        let parts = if completed.parts.is_empty() {
            self.metadata.list_uploaded_parts(upload_id).await?
        } else {
            completed.parts
        };
        let size = self
            .mount
            .assemble_parts(bucket, key, upload_id, &parts)
            .await?;
        self.mount.cleanup_parts(bucket, key, upload_id).await?;
        self.metadata.delete_multipart(upload_id).await?;
        let part_etags: Vec<String> = parts.iter().map(|p| p.etag.clone()).collect();
        let etag = Self::compute_multipart_etag(&part_etags);
        let obj = S3Object {
            bucket: bucket.to_owned(),
            key: key.to_owned(),
            size,
            etag,
            last_modified: chrono::Utc::now(),
            content_type: None,
            content_encoding: None,
            storage_class: upload.storage_class,
            user_metadata: upload.user_metadata,
            user_tags: upload.user_tags,
            version: Self::new_version_ref(),
            parts: parts
                .iter()
                .map(|p| ObjectPart {
                    part_number: p.part_number,
                    etag: p.etag.clone(),
                    size: p.size,
                    checksum: None,
                    last_modified: Some(chrono::Utc::now()),
                })
                .collect(),
            checksums: vec![],
            replication_state: ReplicationState::None,
            retention: None,
            legal_hold: None,
            restore_expiry: None,
            restore_ongoing: false,
        };
        self.metadata.store_object_meta(&obj).await?;
        Ok(obj)
    }

    async fn abort_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> Result<(), S3EngineError> {
        self.metadata
            .load_multipart(upload_id)
            .await?
            .ok_or_else(|| S3EngineError::MultipartNotFound(upload_id.to_owned()))?;
        self.mount.cleanup_parts(bucket, key, upload_id).await?;
        self.metadata.delete_multipart(upload_id).await?;
        Ok(())
    }

    async fn list_multipart_uploads(
        &self,
        bucket: &str,
        options: ListOptions,
    ) -> Result<Vec<MultipartUpload>, S3EngineError> {
        self.metadata
            .load_bucket(bucket)
            .await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        let mut uploads = self.metadata.list_multipart_uploads(bucket).await?;
        if let Some(ref prefix) = options.prefix {
            uploads.retain(|u| u.key.starts_with(prefix.as_str()));
        }
        uploads.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(uploads)
    }
}
