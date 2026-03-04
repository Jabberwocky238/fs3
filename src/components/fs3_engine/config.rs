use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::errors::S3EngineError;
use crate::types::traits::s3_engine::*;
use crate::types::traits::s3_metadata_storage::*;

use super::FS3Engine;

impl<S, M> FS3Engine<S, M>
where
    S: S3MetadataStorageBucket + Send + Sync,
    M: Send + Sync,
{
    async fn ensure_bucket_meta(&self, bucket: &str) -> Result<BucketMetadataBundle, S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(self.metadata.load_bucket_metadata(bucket).await?.unwrap_or_default())
    }
}

#[async_trait]
impl<S, M> S3BucketConfigEngine for FS3Engine<S, M>
where
    S: S3MetadataStorageBucket + Send + Sync,
    M: Send + Sync,
{
    async fn get_bucket_location(&self, bucket: &str) -> Result<String, S3EngineError> {
        let b = self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(b.region.unwrap_or_else(|| "us-east-1".to_owned()))
    }

    async fn get_bucket_metadata(&self, bucket: &str) -> Result<BucketMetadataBundle, S3EngineError> {
        self.ensure_bucket_meta(bucket).await
    }

    async fn put_bucket_metadata(&self, bucket: &str, metadata: BucketMetadataBundle) -> Result<(), S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        self.metadata.store_bucket_metadata(bucket, &metadata).await?;
        Ok(())
    }
}

#[async_trait]
impl<S, M> S3BucketLifecycleEngine for FS3Engine<S, M>
where
    S: S3MetadataStorageBucket + Send + Sync,
    M: Send + Sync,
{
    async fn get_bucket_lifecycle(&self, _bucket: &str) -> Result<Vec<String>, S3EngineError> {
        Ok(vec![])
    }

    async fn put_bucket_lifecycle(&self, bucket: &str, _rules: Vec<String>) -> Result<(), S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(())
    }

    async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<(), S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(())
    }
}

#[async_trait]
impl<S, M> S3BucketEncryptionEngine for FS3Engine<S, M>
where
    S: S3MetadataStorageBucket + Send + Sync,
    M: Send + Sync,
{
    async fn get_bucket_encryption(&self, bucket: &str) -> Result<Option<BucketEncryption>, S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(None)
    }

    async fn put_bucket_encryption(&self, bucket: &str, _algorithm: String, _key_id: Option<String>) -> Result<(), S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(())
    }

    async fn delete_bucket_encryption(&self, bucket: &str) -> Result<(), S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(())
    }
}

#[async_trait]
impl<S, M> S3BucketObjectLockEngine for FS3Engine<S, M>
where
    S: S3MetadataStorageBucket + Send + Sync,
    M: Send + Sync,
{
    async fn get_bucket_object_lock_config(&self, bucket: &str) -> Result<Option<BucketObjectLockConfig>, S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(None)
    }

    async fn put_bucket_object_lock_config(&self, bucket: &str, _enabled: bool, _mode: Option<String>, _days: Option<u32>, _years: Option<u32>) -> Result<(), S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(())
    }
}

#[async_trait]
impl<S, M> S3BucketVersionEngine for FS3Engine<S, M>
where
    S: S3MetadataStorageBucket + Send + Sync,
    M: Send + Sync,
{
    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<BucketVersioning>, S3EngineError> {
        let meta = self.get_bucket_metadata(bucket).await?;
        Ok(meta.versioning_status.map(|status| BucketVersioning {
            status,
            mfa_delete: meta.versioning_mfa_delete,
        }))
    }

    async fn put_bucket_versioning(&self, bucket: &str, status: String, mfa_delete: Option<String>) -> Result<(), S3EngineError> {
        let mut meta = self.get_bucket_metadata(bucket).await?;
        meta.versioning_status = Some(status);
        meta.versioning_mfa_delete = mfa_delete;
        self.put_bucket_metadata(bucket, meta).await
    }
}

#[async_trait]
impl<S, M> S3BucketNotificationEngine for FS3Engine<S, M>
where
    S: S3MetadataStorageBucket + Send + Sync,
    M: Send + Sync,
{
    async fn get_bucket_notification(&self, bucket: &str) -> Result<Vec<String>, S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(vec![])
    }

    async fn put_bucket_notification(&self, bucket: &str, _configs: Vec<String>) -> Result<(), S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(())
    }
}

#[async_trait]
impl<S, M> S3BucketReplicationEngine for FS3Engine<S, M>
where
    S: S3MetadataStorageBucket + Send + Sync,
    M: Send + Sync,
{
    async fn get_bucket_replication(&self, bucket: &str) -> Result<Option<BucketReplication>, S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(None)
    }

    async fn put_bucket_replication(&self, bucket: &str, _role: String, _rules: Vec<String>) -> Result<(), S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(())
    }

    async fn delete_bucket_replication(&self, bucket: &str) -> Result<(), S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(())
    }

    async fn get_bucket_replication_metrics(&self, bucket: &str) -> Result<ReplicationMetrics, S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(ReplicationMetrics { raw_json: "{\"status\":\"ok\"}".to_owned() })
    }

    async fn validate_bucket_replication_creds(&self, bucket: &str) -> Result<ReplicationCredsValidation, S3EngineError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(ReplicationCredsValidation { valid: true, detail: None })
    }
}

#[async_trait]
impl<S, M> S3BucketTaggingEngine for FS3Engine<S, M>
where
    S: S3MetadataStorageBucket + Send + Sync,
    M: Send + Sync,
{
    async fn get_bucket_tagging(&self, bucket: &str) -> Result<Option<std::collections::HashMap<String, String>>, S3EngineError> {
        self.ensure_bucket_meta(bucket).await.map(|m| m.tagging_map)
    }

    async fn put_bucket_tagging(&self, bucket: &str, tags: std::collections::HashMap<String, String>) -> Result<(), S3EngineError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.tagging_map = Some(tags);
        self.metadata.store_bucket_metadata(bucket, &m).await?;
        Ok(())
    }

    async fn delete_bucket_tagging(&self, bucket: &str) -> Result<(), S3EngineError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.tagging_map = None;
        self.metadata.store_bucket_metadata(bucket, &m).await?;
        Ok(())
    }
}
