use async_trait::async_trait;
use crate::types::traits::s3_engine::*;
use crate::types::s3::core::*;
use crate::types::errors::S3EngineError;
use super::FS3Engine;

#[async_trait]
impl S3BucketLifecycleEngine for FS3Engine {
    async fn get_bucket_lifecycle(&self, _bucket: &str) -> Result<Vec<String>, S3EngineError> {
        Ok(vec![r#"<Rule><ID>rule1</ID><Status>Enabled</Status><Expiration><Days>30</Days></Expiration></Rule>"#.to_string()])
    }
    async fn put_bucket_lifecycle(&self, _bucket: &str, _rules: Vec<String>) -> Result<(), S3EngineError> {
        Ok(())
    }
    async fn delete_bucket_lifecycle(&self, _bucket: &str) -> Result<(), S3EngineError> {
        Ok(())
    }
}

#[async_trait]
impl S3BucketEncryptionEngine for FS3Engine {
    async fn get_bucket_encryption(&self, _bucket: &str) -> Result<Option<BucketEncryption>, S3EngineError> {
        Ok(Some(BucketEncryption {
            algorithm: "AES256".to_string(),
            key_id: None,
        }))
    }
    async fn put_bucket_encryption(&self, _bucket: &str, _algorithm: String, _key_id: Option<String>) -> Result<(), S3EngineError> {
        Ok(())
    }
    async fn delete_bucket_encryption(&self, _bucket: &str) -> Result<(), S3EngineError> {
        Ok(())
    }
}

#[async_trait]
impl S3BucketObjectLockEngine for FS3Engine {
    async fn get_bucket_object_lock_config(&self, _bucket: &str) -> Result<Option<BucketObjectLockConfig>, S3EngineError> {
        Ok(None)
    }
    async fn put_bucket_object_lock_config(&self, _bucket: &str, _enabled: bool, _mode: Option<String>, _days: Option<u32>, _years: Option<u32>) -> Result<(), S3EngineError> {
        Ok(())
    }
}

#[async_trait]
impl S3BucketVersionEngine for FS3Engine {
    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<BucketVersioning>, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let status = self.storage.read_bucket_versioning(&ctx, bucket).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))?;

        if let Some(s) = status {
            Ok(Some(BucketVersioning { status: s, mfa_delete: None }))
        } else {
            Ok(None)
        }
    }

    async fn put_bucket_versioning(&self, bucket: &str, status: String, _mfa_delete: Option<String>) -> Result<(), S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        self.storage.write_bucket_versioning(&ctx, bucket, &status).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))
    }
}

#[async_trait]
impl S3BucketNotificationEngine for FS3Engine {
    async fn get_bucket_notification(&self, _bucket: &str) -> Result<Vec<String>, S3EngineError> {
        Ok(Vec::new())
    }
    async fn put_bucket_notification(&self, _bucket: &str, _configs: Vec<String>) -> Result<(), S3EngineError> {
        Ok(())
    }
}

#[async_trait]
impl S3BucketReplicationEngine for FS3Engine {
    async fn get_bucket_replication(&self, _bucket: &str) -> Result<Option<BucketReplication>, S3EngineError> {
        Ok(None)
    }
    async fn put_bucket_replication(&self, _bucket: &str, _role: String, _rules: Vec<String>) -> Result<(), S3EngineError> {
        Ok(())
    }
    async fn delete_bucket_replication(&self, _bucket: &str) -> Result<(), S3EngineError> {
        Ok(())
    }
    async fn get_bucket_replication_metrics(&self, _bucket: &str) -> Result<ReplicationMetrics, S3EngineError> {
        Ok(ReplicationMetrics::default())
    }
    async fn validate_bucket_replication_creds(&self, _bucket: &str) -> Result<ReplicationCredsValidation, S3EngineError> {
        Ok(ReplicationCredsValidation::default())
    }
}

#[async_trait]
impl S3BucketTaggingEngine for FS3Engine {
    async fn get_bucket_tagging(&self, bucket: &str) -> Result<Option<std::collections::HashMap<String, String>>, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let tags_json = self.storage.read_bucket_tags(&ctx, bucket).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))?;

        if let Some(json) = tags_json {
            let tags: std::collections::HashMap<String, String> = serde_json::from_str(&json)
                .map_err(|e| S3EngineError::Storage(e.to_string()))?;
            Ok(Some(tags))
        } else {
            Ok(None)
        }
    }

    async fn put_bucket_tagging(&self, bucket: &str, tags: std::collections::HashMap<String, String>) -> Result<(), S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let json = serde_json::to_string(&tags)
            .map_err(|e| S3EngineError::Storage(e.to_string()))?;
        self.storage.write_bucket_tags(&ctx, bucket, &json).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))
    }

    async fn delete_bucket_tagging(&self, bucket: &str) -> Result<(), S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        self.storage.delete_bucket_tags(&ctx, bucket).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))
    }
}

#[async_trait]
impl S3BucketConfigEngine for FS3Engine {
    async fn get_bucket_location(&self, _bucket: &str) -> Result<String, S3EngineError> {
        Ok("us-east-1".to_string())
    }
    async fn get_bucket_metadata(&self, bucket: &str) -> Result<BucketMetadataBundle, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let cors_json = self.storage.read_bucket_cors(&ctx, bucket).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))?;
        let cors = if let Some(json) = cors_json {
            serde_json::from_str(&json).ok()
        } else {
            None
        };
        Ok(BucketMetadataBundle { cors, ..Default::default() })
    }
    async fn put_bucket_metadata(&self, _bucket: &str, _metadata: BucketMetadataBundle) -> Result<(), S3EngineError> {
        Ok(())
    }
}
