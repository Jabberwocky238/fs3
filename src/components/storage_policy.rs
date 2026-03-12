use async_trait::async_trait;
use std::sync::Arc;

use crate::types::FS3Error;
use crate::types::s3::object_layer_types::Context;
use crate::types::traits::s3_policyengine::*;
use crate::types::traits::storage_api::StorageAPI;

pub struct StoragePolicyEngine {
    storage: Arc<dyn StorageAPI<FS3Error>>,
}

impl StoragePolicyEngine {
    pub fn new(storage: Arc<dyn StorageAPI<FS3Error>>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl S3IamPolicyEngine<FS3Error> for StoragePolicyEngine {
    async fn is_allowed(&self, _ctx: &PolicyEvalContext) -> Result<PolicyEffect, FS3Error> {
        Ok(PolicyEffect::Allow)
    }

    async fn get_user_policies(&self, _identity: &str) -> Result<Vec<String>, FS3Error> {
        Ok(Vec::new())
    }

    async fn get_group_policies(&self, _group: &str) -> Result<Vec<String>, FS3Error> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl S3BucketPolicyEngine<FS3Error> for StoragePolicyEngine {
    async fn is_allowed(&self, _bucket: &str, _ctx: &PolicyEvalContext) -> Result<PolicyEffect, FS3Error> {
        Ok(PolicyEffect::Allow)
    }

    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<String>, FS3Error> {
        let ctx = Context { request_id: "".to_string() };
        self.storage.read_bucket_policy(&ctx, bucket).await
    }

    async fn put_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<(), FS3Error> {
        serde_json::from_str::<serde_json::Value>(policy_json)?;
        let ctx = Context { request_id: "".to_string() };
        self.storage.write_bucket_policy(&ctx, bucket, policy_json).await
    }

    async fn delete_bucket_policy(&self, bucket: &str) -> Result<(), FS3Error> {
        let ctx = Context { request_id: "".to_string() };
        self.storage.delete_bucket_policy(&ctx, bucket).await
    }
}

#[async_trait]
impl S3PolicyEngine<FS3Error> for StoragePolicyEngine {
    async fn check_access(&self, _ctx: &PolicyEvalContext) -> Result<PolicyEffect, FS3Error> {
        Ok(PolicyEffect::Allow)
    }
}
