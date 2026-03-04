use async_trait::async_trait;
use std::sync::Arc;
use crate::types::traits::s3_policyengine::*;
use crate::types::traits::storage_api::StorageAPI;
use crate::types::s3::object_layer_types::Context;

pub struct StoragePolicyEngine {
    storage: Arc<dyn StorageAPI>,
}

impl StoragePolicyEngine {
    pub fn new(storage: Arc<dyn StorageAPI>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl S3IamPolicyEngine for StoragePolicyEngine {
    async fn is_allowed(&self, _ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        Ok(PolicyEffect::Allow)
    }
    async fn get_user_policies(&self, _identity: &str) -> Result<Vec<String>, PolicyEngineError> {
        Ok(Vec::new())
    }
    async fn get_group_policies(&self, _group: &str) -> Result<Vec<String>, PolicyEngineError> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl S3BucketPolicyEngine for StoragePolicyEngine {
    async fn is_allowed(&self, _bucket: &str, _ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        Ok(PolicyEffect::Allow)
    }

    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<String>, PolicyEngineError> {
        let ctx = Context { request_id: "".to_string() };
        self.storage.read_bucket_policy(&ctx, bucket).await
            .map_err(|e| PolicyEngineError::Storage(e.to_string()))
    }

    async fn put_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<(), PolicyEngineError> {
        let ctx = Context { request_id: "".to_string() };
        self.storage.write_bucket_policy(&ctx, bucket, policy_json).await
            .map_err(|e| PolicyEngineError::Storage(e.to_string()))
    }

    async fn delete_bucket_policy(&self, bucket: &str) -> Result<(), PolicyEngineError> {
        let ctx = Context { request_id: "".to_string() };
        self.storage.delete_bucket_policy(&ctx, bucket).await
            .map_err(|e| PolicyEngineError::Storage(e.to_string()))
    }
}

#[async_trait]
impl S3PolicyEngine for StoragePolicyEngine {
    async fn check_access(&self, _ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        Ok(PolicyEffect::Allow)
    }
}
