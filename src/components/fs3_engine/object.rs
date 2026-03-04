use async_trait::async_trait;
use crate::types::traits::s3_engine::*;
use crate::types::s3::core::*;
use crate::types::errors::S3EngineError;
use super::FS3Engine;

#[async_trait]
impl S3ObjectEngine for FS3Engine {
    async fn head_object(&self, _bucket: &str, _key: &str, _options: ObjectReadOptions) -> Result<S3Object, S3EngineError> {
        Err(S3EngineError::Storage("not implemented".to_string()))
    }

    async fn get_object(&self, _bucket: &str, _key: &str, _options: ObjectReadOptions) -> Result<(S3Object, BoxByteStream), S3EngineError> {
        Err(S3EngineError::Storage("not implemented".to_string()))
    }

    async fn put_object(&self, _bucket: &str, _key: &str, _body: BoxByteStream, _options: ObjectWriteOptions) -> Result<S3Object, S3EngineError> {
        Err(S3EngineError::Storage("not implemented".to_string()))
    }

    async fn copy_object(&self, _src_bucket: &str, _src_key: &str, _dst_bucket: &str, _dst_key: &str, _options: ObjectWriteOptions) -> Result<S3Object, S3EngineError> {
        Err(S3EngineError::Storage("not implemented".to_string()))
    }

    async fn delete_object(&self, _bucket: &str, _key: &str, _options: DeleteObjectOptions) -> Result<ObjectVersionRef, S3EngineError> {
        Err(S3EngineError::Storage("not implemented".to_string()))
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
