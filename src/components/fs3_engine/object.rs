use async_trait::async_trait;
use crate::types::traits::s3_engine::*;
use crate::types::s3::core::*;

use super::FS3Engine;
// use futures::TryStreamExt;

#[async_trait]
impl S3ObjectEngine for FS3Engine {
    async fn put_object(&self, bucket: &str, key: &str, body: BoxByteStream, options: ObjectWriteOptions) -> Result<S3Object, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };

        let data = crate::types::s3::storage_types::PutObjReader { reader: body, size: options.size as i64 };
        let opts = crate::types::s3::object_layer_types::ObjectOptions {
            version_id: None,
            user_defined: options.user_metadata.clone(),
            range: None,
        };

        let info = self.object_layer.put_object(&ctx, bucket, key, data, opts).await
            .map_err(|e| S3EngineError::from(e.to_string()))?;

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

    async fn head_object(&self, bucket: &str, key: &str, options: ObjectReadOptions) -> Result<S3Object, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let opts = crate::types::s3::object_layer_types::ObjectOptions { 
            version_id: options.version_id, user_defined: Default::default(), range: None 
        };

        let info = self.object_layer.get_object_info(&ctx, bucket, key, opts).await
            .map_err(|e| S3EngineError::from(e.to_string()))?;

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

    async fn get_object(&self, bucket: &str, key: &str, options: ObjectReadOptions) -> Result<(S3Object, BoxByteStream), S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let opts = crate::types::s3::object_layer_types::ObjectOptions {
            version_id: options.version_id,
            user_defined: Default::default(),
            range: options.range,
        };

        let (info, stream) = self.object_layer.get_object(&ctx, bucket, key, opts).await
            .map_err(|e| S3EngineError::from(e.to_string()))?;

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

    async fn copy_object(&self, src_bucket: &str, src_key: &str, dst_bucket: &str, dst_key: &str, options: ObjectWriteOptions) -> Result<S3Object, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let src_opts = crate::types::s3::object_layer_types::ObjectOptions { 
            version_id: None, user_defined: Default::default(), range: None 
        };
        let dst_opts = crate::types::s3::object_layer_types::ObjectOptions { 
            version_id: None, user_defined: options.user_metadata.clone(), range: None 
        };

        let src_info = self.object_layer.get_object_info(&ctx, src_bucket, src_key, src_opts.clone()).await
            .map_err(|e| S3EngineError::from(e.to_string()))?;

        let info = self.object_layer.copy_object(&ctx, src_bucket, src_key, dst_bucket, dst_key, src_info.clone(), src_opts, dst_opts).await
            .map_err(|e| S3EngineError::from(e.to_string()))?;

        Ok(S3Object {
            bucket: dst_bucket.to_string(),
            key: dst_key.to_string(),
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

    async fn delete_object(&self, bucket: &str, key: &str, options: DeleteObjectOptions) -> Result<ObjectVersionRef, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let opts = crate::types::s3::object_layer_types::ObjectOptions { 
            version_id: options.version_id, 
            user_defined: Default::default(),
            range: None
        };

        self.object_layer.delete_object(&ctx, bucket, key, opts).await
            .map_err(|e| S3EngineError::from(e.to_string()))?;

        Ok(ObjectVersionRef { version_id: None, is_latest: true, delete_marker: false })
    }

    async fn delete_objects(&self, bucket: &str, keys: Vec<String>, options: DeleteObjectOptions) -> Result<DeleteResult, S3EngineError> {
        let mut deleted = Vec::new();
        let mut errors = Vec::new();

        for key in keys {
            match self.delete_object(bucket, &key, options.clone()).await {
                Ok(version_ref) => deleted.push(version_ref),
                Err(e) => errors.push(crate::types::s3::core::S3ErrorDetail {
                    key: Some(key.clone()),
                    code: "InternalError".to_string(),
                    message: e.to_string(),
                    version_id: None,
                }),
            }
        }

        Ok(DeleteResult { deleted, errors })
    }
}

#[async_trait]
impl S3ObjectTaggingEngine for FS3Engine {
    async fn get_object_tagging(&self, bucket: &str, key: &str) -> Result<TagMap, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let json = self.storage.read_object_tags(&ctx, bucket, key).await
            .map_err(|e| S3EngineError::from(e.to_string()))?;
        if let Some(j) = json {
            serde_json::from_str(&j).map_err(|e| S3EngineError::from(e.to_string()))
        } else {
            Ok(TagMap::default())
        }
    }

    async fn put_object_tagging(&self, bucket: &str, key: &str, tags: TagMap) -> Result<(), S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let json = serde_json::to_string(&tags).map_err(|e| S3EngineError::from(e.to_string()))?;
        self.storage.write_object_tags(&ctx, bucket, key, &json).await
            .map_err(|e| S3EngineError::from(e.to_string()))
    }

    async fn delete_object_tagging(&self, bucket: &str, key: &str) -> Result<(), S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        self.storage.delete_object_tags(&ctx, bucket, key).await
            .map_err(|e| S3EngineError::from(e.to_string()))
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

