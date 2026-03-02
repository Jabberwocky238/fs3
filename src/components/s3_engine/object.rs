use async_trait::async_trait;
use futures::TryStreamExt;

use crate::types::s3::core::*;
use crate::types::errors::S3EngineError;
use crate::types::traits::s3_engine::S3ObjectEngine;
use crate::types::traits::s3_metadata_storage::*;
use crate::types::traits::s3_mount::*;

use super::S3EngineImpl;

#[async_trait]
impl<S, M> S3ObjectEngine for S3EngineImpl<S, M>
where
    S: S3MetadataStorageBucket
        + S3MetadataStorageObject
        + Send + Sync,
    M: S3MountRead
        + S3MountWrite
        + Send + Sync,
{
    async fn head_object(&self, bucket: &str, key: &str, _options: ObjectReadOptions) -> Result<S3Object, S3EngineError> {
        self.metadata.load_object_meta(bucket, key).await?
            .ok_or_else(|| S3EngineError::ObjectNotFound { bucket: bucket.to_owned(), key: key.to_owned() })
    }

    async fn get_object(&self, bucket: &str, key: &str, options: ObjectReadOptions) -> Result<(S3Object, BoxByteStream), S3EngineError> {
        let obj = self.head_object(bucket, key, options.clone()).await?;
        let stream = if let Some(ref range) = options.range {
            self.mount.read_object_range(bucket, key, range).await?
        } else {
            self.mount.read_object(bucket, key).await?
        };
        Ok((obj, stream))
    }

    async fn put_object(&self, bucket: &str, key: &str, body: BoxByteStream, options: ObjectWriteOptions) -> Result<S3Object, S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        let chunks: Vec<bytes::Bytes> = body.try_collect().await
            .map_err(|e| S3EngineError::Storage(format!("stream error: {e}")))?;
        let mut buf = Vec::new();
        for chunk in &chunks { buf.extend_from_slice(chunk); }
        let etag = Self::compute_etag(&buf);
        let size = buf.len() as u64;
        let stream: BoxByteStream = Box::pin(futures::stream::once(async { Ok(bytes::Bytes::from(buf)) }));
        self.mount.write_object(bucket, key, stream).await?;
        let obj = S3Object {
            bucket: bucket.to_owned(), key: key.to_owned(), size, etag,
            last_modified: chrono::Utc::now(),
            content_type: options.content_type.clone(),
            content_encoding: options.content_encoding.clone(),
            storage_class: options.storage_class.clone(),
            user_metadata: options.user_metadata.clone(),
            user_tags: options.user_tags.clone(),
            version: Self::new_version_ref(),
            parts: vec![],
            checksums: options.checksum.into_iter().collect(),
            replication_state: ReplicationState::None,
            retention: options.retention.clone(),
            legal_hold: options.legal_hold.clone(),
            restore_expiry: None, restore_ongoing: false,
        };
        self.metadata.store_object_meta(&obj).await?;
        Ok(obj)
    }

    async fn delete_object(&self, bucket: &str, key: &str, _options: DeleteObjectOptions) -> Result<ObjectVersionRef, S3EngineError> {
        let obj = self.metadata.load_object_meta(bucket, key).await?
            .ok_or_else(|| S3EngineError::ObjectNotFound { bucket: bucket.to_owned(), key: key.to_owned() })?;
        self.metadata.delete_object_meta(bucket, key).await?;
        self.mount.delete_object(bucket, key).await?;
        Ok(obj.version)
    }

    async fn delete_objects(&self, bucket: &str, keys: Vec<String>, options: DeleteObjectOptions) -> Result<DeleteResult, S3EngineError> {
        let mut deleted = Vec::new();
        let mut errors = Vec::new();
        for key in keys {
            match self.delete_object(bucket, &key, options.clone()).await {
                Ok(v) => deleted.push(v),
                Err(e) => errors.push(S3ErrorDetail {
                    code: "DeleteFailed".to_owned(), message: e.to_string(),
                    key: Some(key), version_id: options.version_id.clone(),
                }),
            }
        }
        Ok(DeleteResult { deleted, errors })
    }

    async fn copy_object(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str, options: ObjectWriteOptions) -> Result<S3Object, S3EngineError> {
        self.metadata.load_bucket(dst_bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(dst_bucket.to_owned()))?;
        let src_obj = self.metadata.load_object_meta(src_bucket, src_key).await?
            .ok_or_else(|| S3EngineError::ObjectNotFound { bucket: src_bucket.to_owned(), key: src_key.to_owned() })?;
        let size = self.mount.copy_object(src_bucket, src_key, dst_bucket, dst_key).await?;
        let obj = S3Object {
            bucket: dst_bucket.to_owned(), key: dst_key.to_owned(), size,
            etag: src_obj.etag.clone(),
            last_modified: chrono::Utc::now(),
            content_type: options.content_type.clone(),
            content_encoding: options.content_encoding.clone(),
            storage_class: options.storage_class.clone(),
            user_metadata: options.user_metadata.clone(),
            user_tags: options.user_tags.clone(),
            version: Self::new_version_ref(),
            parts: vec![],
            checksums: options.checksum.into_iter().collect(),
            replication_state: ReplicationState::None,
            retention: options.retention.clone(),
            legal_hold: options.legal_hold.clone(),
            restore_expiry: None, restore_ongoing: false,
        };
        self.metadata.store_object_meta(&obj).await?;
        Ok(obj)
    }

    async fn get_object_tagging(&self, bucket: &str, key: &str) -> Result<TagMap, S3EngineError> {
        Ok(self.head_object(bucket, key, ObjectReadOptions::default()).await?.user_tags)
    }

    async fn put_object_tagging(&self, bucket: &str, key: &str, tags: TagMap) -> Result<(), S3EngineError> {
        let mut obj = self.head_object(bucket, key, ObjectReadOptions::default()).await?;
        obj.user_tags = tags;
        self.metadata.store_object_meta(&obj).await?;
        Ok(())
    }

    async fn delete_object_tagging(&self, bucket: &str, key: &str) -> Result<(), S3EngineError> {
        self.put_object_tagging(bucket, key, TagMap::new()).await
    }

    async fn get_object_retention(&self, bucket: &str, key: &str) -> Result<Option<ObjectRetention>, S3EngineError> {
        Ok(self.head_object(bucket, key, ObjectReadOptions::default()).await?.retention)
    }

    async fn put_object_retention(&self, bucket: &str, key: &str, retention: ObjectRetention) -> Result<(), S3EngineError> {
        let mut obj = self.head_object(bucket, key, ObjectReadOptions::default()).await?;
        obj.retention = Some(retention);
        self.metadata.store_object_meta(&obj).await?;
        Ok(())
    }

    async fn get_object_legal_hold(&self, bucket: &str, key: &str) -> Result<Option<ObjectLegalHold>, S3EngineError> {
        Ok(self.head_object(bucket, key, ObjectReadOptions::default()).await?.legal_hold)
    }

    async fn put_object_legal_hold(&self, bucket: &str, key: &str, legal_hold: ObjectLegalHold) -> Result<(), S3EngineError> {
        let mut obj = self.head_object(bucket, key, ObjectReadOptions::default()).await?;
        obj.legal_hold = Some(legal_hold);
        self.metadata.store_object_meta(&obj).await?;
        Ok(())
    }
}
