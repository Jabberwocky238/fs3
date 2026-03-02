use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::traits::s3_engine::S3BucketConfigEngine;
use crate::types::traits::s3_metadata_storage::*;

use super::{S3EngineImpl, S3EngineImplError};

impl<S, M> S3EngineImpl<S, M>
where
    S: S3MetadataStorageBucket<S3EngineImplError> + Send + Sync,
    M: Send + Sync,
{
    async fn ensure_bucket_meta(&self, bucket: &str) -> Result<BucketMetadataBundle, S3EngineImplError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineImplError::BucketNotFound(bucket.to_owned()))?;
        Ok(self.metadata.load_bucket_metadata(bucket).await?.unwrap_or_default())
    }
}

#[async_trait]
impl<S, M> S3BucketConfigEngine<S3EngineImplError> for S3EngineImpl<S, M>
where
    S: S3MetadataStorageBucket<S3EngineImplError> + Send + Sync,
    M: Send + Sync,
{
    async fn get_bucket_location(&self, bucket: &str) -> Result<String, S3EngineImplError> {
        let b = self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineImplError::BucketNotFound(bucket.to_owned()))?;
        Ok(b.region.unwrap_or_else(|| "us-east-1".to_owned()))
    }

    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineImplError> {
        self.ensure_bucket_meta(bucket).await.map(|m| m.policy_json)
    }
    async fn put_bucket_policy(&self, bucket: &str, policy_json: String) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.policy_json = Some(Self::now_doc(policy_json));
        self.metadata.store_bucket_metadata(bucket, &m).await
    }
    async fn delete_bucket_policy(&self, bucket: &str) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.policy_json = None;
        self.metadata.store_bucket_metadata(bucket, &m).await
    }
    async fn get_bucket_policy_status(&self, bucket: &str) -> Result<BucketPolicyStatus, S3EngineImplError> {
        let m = self.ensure_bucket_meta(bucket).await?;
        let is_public = m.policy_json.as_ref()
            .map(|d| d.body.to_ascii_lowercase().contains("\"effect\":\"allow\""))
            .unwrap_or(false);
        Ok(BucketPolicyStatus { is_public })
    }

    async fn get_bucket_lifecycle(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineImplError> {
        self.ensure_bucket_meta(bucket).await.map(|m| m.lifecycle_xml)
    }
    async fn put_bucket_lifecycle(&self, bucket: &str, lifecycle_xml: String) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.lifecycle_xml = Some(Self::now_doc(lifecycle_xml));
        self.metadata.store_bucket_metadata(bucket, &m).await
    }
    async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.lifecycle_xml = None;
        self.metadata.store_bucket_metadata(bucket, &m).await
    }

    async fn get_bucket_encryption(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineImplError> {
        self.ensure_bucket_meta(bucket).await.map(|m| m.encryption_xml)
    }
    async fn put_bucket_encryption(&self, bucket: &str, encryption_xml: String) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.encryption_xml = Some(Self::now_doc(encryption_xml));
        self.metadata.store_bucket_metadata(bucket, &m).await
    }
    async fn delete_bucket_encryption(&self, bucket: &str) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.encryption_xml = None;
        self.metadata.store_bucket_metadata(bucket, &m).await
    }

    async fn get_bucket_object_lock_config(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineImplError> {
        self.ensure_bucket_meta(bucket).await.map(|m| m.object_lock_xml)
    }
    async fn put_bucket_object_lock_config(&self, bucket: &str, object_lock_xml: String) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.object_lock_xml = Some(Self::now_doc(object_lock_xml));
        self.metadata.store_bucket_metadata(bucket, &m).await
    }

    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineImplError> {
        self.ensure_bucket_meta(bucket).await.map(|m| m.versioning_xml)
    }
    async fn put_bucket_versioning(&self, bucket: &str, versioning_xml: String) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.versioning_xml = Some(Self::now_doc(versioning_xml));
        self.metadata.store_bucket_metadata(bucket, &m).await
    }

    async fn get_bucket_notification(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineImplError> {
        self.ensure_bucket_meta(bucket).await.map(|m| m.notification_xml)
    }
    async fn put_bucket_notification(&self, bucket: &str, notification_xml: String) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.notification_xml = Some(Self::now_doc(notification_xml));
        self.metadata.store_bucket_metadata(bucket, &m).await
    }

    async fn get_bucket_replication(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineImplError> {
        self.ensure_bucket_meta(bucket).await.map(|m| m.replication_xml)
    }
    async fn put_bucket_replication(&self, bucket: &str, replication_xml: String) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.replication_xml = Some(Self::now_doc(replication_xml));
        self.metadata.store_bucket_metadata(bucket, &m).await
    }
    async fn delete_bucket_replication(&self, bucket: &str) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.replication_xml = None;
        self.metadata.store_bucket_metadata(bucket, &m).await
    }

    async fn get_bucket_tagging(&self, bucket: &str) -> Result<Option<TimedDocument>, S3EngineImplError> {
        self.ensure_bucket_meta(bucket).await.map(|m| m.tagging_xml)
    }
    async fn put_bucket_tagging(&self, bucket: &str, tagging_xml: String) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.tagging_xml = Some(Self::now_doc(tagging_xml));
        self.metadata.store_bucket_metadata(bucket, &m).await
    }
    async fn delete_bucket_tagging(&self, bucket: &str) -> Result<(), S3EngineImplError> {
        let mut m = self.ensure_bucket_meta(bucket).await?;
        m.tagging_xml = None;
        self.metadata.store_bucket_metadata(bucket, &m).await
    }

    async fn get_bucket_metadata(&self, bucket: &str) -> Result<BucketMetadataBundle, S3EngineImplError> {
        self.ensure_bucket_meta(bucket).await
    }
    async fn put_bucket_metadata(&self, bucket: &str, metadata: BucketMetadataBundle) -> Result<(), S3EngineImplError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineImplError::BucketNotFound(bucket.to_owned()))?;
        self.metadata.store_bucket_metadata(bucket, &metadata).await
    }

    async fn get_bucket_replication_metrics(&self, bucket: &str) -> Result<ReplicationMetrics, S3EngineImplError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineImplError::BucketNotFound(bucket.to_owned()))?;
        Ok(ReplicationMetrics { raw_json: "{\"status\":\"ok\"}".to_owned() })
    }
    async fn validate_bucket_replication_creds(&self, bucket: &str) -> Result<ReplicationCredsValidation, S3EngineImplError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineImplError::BucketNotFound(bucket.to_owned()))?;
        Ok(ReplicationCredsValidation { valid: true, detail: None })
    }
}