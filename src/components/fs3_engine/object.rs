use async_trait::async_trait;
use crate::types::traits::s3_engine::*;
use crate::types::s3::core::*;
use crate::types::errors::S3EngineError;
use super::FS3Engine;

#[async_trait]
impl S3ObjectEngine for FS3Engine {
    async fn put_object(&self, bucket: &str, key: &str, body: BoxByteStream, options: ObjectWriteOptions) -> Result<S3Object, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };

        let data = crate::types::s3::storage_types::PutObjReader { reader: body, size: 0 };
        let opts = crate::types::s3::object_layer_types::ObjectOptions {
            version_id: None,
            user_defined: options.user_metadata.clone(),
        };

        let info = self.object_layer.put_object(&ctx, bucket, key, data, opts).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))?;

        Ok(S3Object {
            bucket: bucket.to_string(),
            key: key.to_string(),
            size: info.size,
            etag: info.etag,
            last_modified: chrono::Utc::now(),
            content_type: Some(info.content_type),
            content_encoding: None,
            storage_class: StorageClass::Standard,
            user_metadata: info.user_defined,
            user_tags: Default::default(),
            version: ObjectVersionRef { version_id: None, is_latest: true, delete_marker: false },
            parts: Vec::new(),
            checksums: Vec::new(),
            replication_state: ReplicationState::default(),
            retention: None,
            legal_hold: None,
            restore_expiry: None,
            restore_ongoing: false,
        })
    }

    async fn head_object(&self, bucket: &str, key: &str, _options: ObjectReadOptions) -> Result<S3Object, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let opts = crate::types::s3::object_layer_types::ObjectOptions { version_id: None, user_defined: Default::default() };

        let info = self.object_layer.get_object_info(&ctx, bucket, key, opts).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))?;

        Ok(S3Object {
            bucket: bucket.to_string(),
            key: key.to_string(),
            size: info.size,
            etag: info.etag,
            last_modified: chrono::Utc::now(),
            content_type: Some(info.content_type),
            content_encoding: None,
            storage_class: StorageClass::Standard,
            user_metadata: info.user_defined,
            user_tags: Default::default(),
            version: ObjectVersionRef { version_id: None, is_latest: true, delete_marker: false },
            parts: Vec::new(),
            checksums: Vec::new(),
            replication_state: ReplicationState::default(),
            retention: None,
            legal_hold: None,
            restore_expiry: None,
            restore_ongoing: false,
        })
    }

    async fn get_object(&self, bucket: &str, key: &str, _options: ObjectReadOptions) -> Result<(S3Object, BoxByteStream), S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let opts = crate::types::s3::object_layer_types::ObjectOptions { version_id: None, user_defined: Default::default() };

        let (info, stream) = self.object_layer.get_object(&ctx, bucket, key, opts).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))?;

        let obj = S3Object {
            bucket: bucket.to_string(),
            key: key.to_string(),
            size: info.size,
            etag: info.etag,
            last_modified: chrono::Utc::now(),
            content_type: Some(info.content_type),
            content_encoding: None,
            storage_class: StorageClass::Standard,
            user_metadata: info.user_defined,
            user_tags: Default::default(),
            version: ObjectVersionRef { version_id: None, is_latest: true, delete_marker: false },
            parts: Vec::new(),
            checksums: Vec::new(),
            replication_state: ReplicationState::default(),
            retention: None,
            legal_hold: None,
            restore_expiry: None,
            restore_ongoing: false,
        };

        Ok((obj, stream))
    }

    async fn copy_object(&self, _src_bucket: &str, _src_key: &str, _dst_bucket: &str, _dst_key: &str, _options: ObjectWriteOptions) -> Result<S3Object, S3EngineError> {
        Err(S3EngineError::Storage("not implemented".to_string()))
    }

    async fn delete_object(&self, bucket: &str, key: &str, _options: DeleteObjectOptions) -> Result<ObjectVersionRef, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let opts = crate::types::s3::object_layer_types::ObjectOptions { version_id: None, user_defined: Default::default() };

        self.object_layer.delete_object(&ctx, bucket, key, opts).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))?;

        Ok(ObjectVersionRef { version_id: None, is_latest: true, delete_marker: false })
    }

    async fn delete_objects(&self, _bucket: &str, _keys: Vec<String>, _options: DeleteObjectOptions) -> Result<DeleteResult, S3EngineError> {
        Err(S3EngineError::Storage("not implemented".to_string()))
    }
}

#[async_trait]
impl S3ObjectTaggingEngine for FS3Engine {
    async fn get_object_tagging(&self, _bucket: &str, _key: &str) -> Result<TagMap, S3EngineError> {
        Ok(TagMap::default())
    }

    async fn put_object_tagging(&self, _bucket: &str, _key: &str, _tags: TagMap) -> Result<(), S3EngineError> {
        Ok(())
    }

    async fn delete_object_tagging(&self, _bucket: &str, _key: &str) -> Result<(), S3EngineError> {
        Ok(())
    }
}

#[async_trait]
impl S3ObjectRetentionEngine for FS3Engine {
    async fn get_object_retention(&self, _bucket: &str, _key: &str) -> Result<Option<ObjectRetention>, S3EngineError> {
        Ok(None)
    }

    async fn put_object_retention(&self, _bucket: &str, _key: &str, _retention: ObjectRetention) -> Result<(), S3EngineError> {
        Ok(())
    }
}

#[async_trait]
impl S3ObjectLegalHoldEngine for FS3Engine {
    async fn get_object_legal_hold(&self, _bucket: &str, _key: &str) -> Result<Option<ObjectLegalHold>, S3EngineError> {
        Ok(None)
    }

    async fn put_object_legal_hold(&self, _bucket: &str, _key: &str, _legal_hold: ObjectLegalHold) -> Result<(), S3EngineError> {
        Ok(())
    }
}
