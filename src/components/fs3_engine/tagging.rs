use async_trait::async_trait;
use std::collections::HashMap;
use crate::types::traits::s3_engine::S3ObjectTaggingEngine;
use crate::types::errors::S3EngineError;
use super::FS3Engine;

#[async_trait]
impl S3ObjectTaggingEngine for FS3Engine {
    async fn get_object_tagging(&self, _bucket: &str, _key: &str) -> Result<HashMap<String, String>, S3EngineError> {
        Ok(self.tags.lock().unwrap().clone())
    }

    async fn put_object_tagging(&self, _bucket: &str, _key: &str, tags: HashMap<String, String>) -> Result<(), S3EngineError> {
        *self.tags.lock().unwrap() = tags;
        Ok(())
    }

    async fn delete_object_tagging(&self, _bucket: &str, _key: &str) -> Result<(), S3EngineError> {
        self.tags.lock().unwrap().clear();
        Ok(())
    }
}
