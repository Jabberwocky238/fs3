use async_trait::async_trait;

use crate::components::s3_engine::memory::{MemoryS3Engine, MemoryS3EngineError};
use crate::types::s3::core::*;
use crate::types::traits::s3_engine::S3BucketConfigEngine;

#[async_trait]
impl S3BucketConfigEngine<MemoryS3EngineError> for MemoryS3Engine {

    async fn get_bucket_location(&self, bucket: &str) -> Result<String, MemoryS3EngineError> {
        let state = self.state.read().await;
        let b = state
            .buckets
            .get(bucket)
            .ok_or_else(|| MemoryS3EngineError::BucketNotFound(bucket.to_owned()))?;
        Ok(b.region.clone().unwrap_or_else(|| "us-east-1".to_owned()))
    }

    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<TimedDocument>, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        Ok(state.bucket_metadata.get(bucket).and_then(|m| m.policy_json.clone()))
    }

    async fn put_bucket_policy(&self, bucket: &str, policy_json: String) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().policy_json = Some(MemoryS3Engine::now_doc(policy_json));
        Ok(())
    }

    async fn delete_bucket_policy(&self, bucket: &str) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().policy_json = None;
        Ok(())
    }

    async fn get_bucket_policy_status(&self, bucket: &str) -> Result<BucketPolicyStatus, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        let is_public = state
            .bucket_metadata
            .get(bucket)
            .and_then(|m| m.policy_json.as_ref())
            .map(|d| d.body.to_ascii_lowercase().contains("\"effect\":\"allow\""))
            .unwrap_or(false);
        Ok(BucketPolicyStatus { is_public })
    }

    async fn get_bucket_lifecycle(&self, bucket: &str) -> Result<Option<TimedDocument>, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        Ok(state.bucket_metadata.get(bucket).and_then(|m| m.lifecycle_xml.clone()))
    }

    async fn put_bucket_lifecycle(&self, bucket: &str, lifecycle_xml: String) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().lifecycle_xml = Some(MemoryS3Engine::now_doc(lifecycle_xml));
        Ok(())
    }

    async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().lifecycle_xml = None;
        Ok(())
    }

    async fn get_bucket_encryption(&self, bucket: &str) -> Result<Option<TimedDocument>, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        Ok(state.bucket_metadata.get(bucket).and_then(|m| m.encryption_xml.clone()))
    }

    async fn put_bucket_encryption(&self, bucket: &str, encryption_xml: String) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().encryption_xml = Some(MemoryS3Engine::now_doc(encryption_xml));
        Ok(())
    }

    async fn delete_bucket_encryption(&self, bucket: &str) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().encryption_xml = None;
        Ok(())
    }

    async fn get_bucket_object_lock_config(&self, bucket: &str) -> Result<Option<TimedDocument>, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        Ok(state.bucket_metadata.get(bucket).and_then(|m| m.object_lock_xml.clone()))
    }

    async fn put_bucket_object_lock_config(&self, bucket: &str, object_lock_xml: String) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().object_lock_xml = Some(MemoryS3Engine::now_doc(object_lock_xml));
        Ok(())
    }

    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<TimedDocument>, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        Ok(state.bucket_metadata.get(bucket).and_then(|m| m.versioning_xml.clone()))
    }

    async fn put_bucket_versioning(&self, bucket: &str, versioning_xml: String) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().versioning_xml = Some(MemoryS3Engine::now_doc(versioning_xml));
        Ok(())
    }

    async fn get_bucket_notification(&self, bucket: &str) -> Result<Option<TimedDocument>, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        Ok(state.bucket_metadata.get(bucket).and_then(|m| m.notification_xml.clone()))
    }

    async fn put_bucket_notification(&self, bucket: &str, notification_xml: String) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().notification_xml = Some(MemoryS3Engine::now_doc(notification_xml));
        Ok(())
    }

    async fn get_bucket_replication(&self, bucket: &str) -> Result<Option<TimedDocument>, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        Ok(state.bucket_metadata.get(bucket).and_then(|m| m.replication_xml.clone()))
    }

    async fn put_bucket_replication(&self, bucket: &str, replication_xml: String) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().replication_xml = Some(MemoryS3Engine::now_doc(replication_xml));
        Ok(())
    }

    async fn delete_bucket_replication(&self, bucket: &str) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().replication_xml = None;
        Ok(())
    }

    async fn get_bucket_tagging(&self, bucket: &str) -> Result<Option<TimedDocument>, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        Ok(state.bucket_metadata.get(bucket).and_then(|m| m.tagging_xml.clone()))
    }

    async fn put_bucket_tagging(&self, bucket: &str, tagging_xml: String) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().tagging_xml = Some(MemoryS3Engine::now_doc(tagging_xml));
        Ok(())
    }

    async fn delete_bucket_tagging(&self, bucket: &str) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.entry(bucket.to_owned()).or_default().tagging_xml = None;
        Ok(())
    }

    async fn get_bucket_metadata(&self, bucket: &str) -> Result<BucketMetadataBundle, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        Ok(state.bucket_metadata.get(bucket).cloned().unwrap_or_default())
    }

    async fn put_bucket_metadata(&self, bucket: &str, metadata: BucketMetadataBundle) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;
        state.bucket_metadata.insert(bucket.to_owned(), metadata);
        Ok(())
    }

    async fn get_bucket_replication_metrics(&self, bucket: &str) -> Result<ReplicationMetrics, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        Ok(ReplicationMetrics { raw_json: "{\"status\":\"ok\",\"engine\":\"memory\"}".to_owned() })
    }

    async fn validate_bucket_replication_creds(
        &self,
        bucket: &str,
    ) -> Result<ReplicationCredsValidation, MemoryS3EngineError> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        Ok(ReplicationCredsValidation {
            valid: true,
            detail: Some("memory engine always treats replication creds as valid".to_owned()),
        })
    }
}
