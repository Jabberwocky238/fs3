use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;

use crate::types::traits::s3_policyengine::*;
use super::policy_doc::PolicyDocument;

/// 桶策略引擎：内存存储 + JSON 策略评估
pub struct FS3BucketPolicyEngine {
    policies: RwLock<HashMap<String, String>>,
}

impl FS3BucketPolicyEngine {
    pub fn new() -> Self {
        Self { policies: RwLock::new(HashMap::new()) }
    }
}

#[async_trait]
impl S3BucketPolicyEngine for FS3BucketPolicyEngine {
    async fn is_allowed(&self, bucket: &str, ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        let json = {
            let map = self.policies.read().map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
            match map.get(bucket) {
                Some(j) => j.clone(),
                None => return Ok(PolicyEffect::Allow), // 无桶策略时默认允许
            }
        };
        let doc = PolicyDocument::parse(&json)
            .map_err(|e| PolicyEngineError::InvalidPolicy(e.to_string()))?;
        Ok(doc.evaluate(ctx).unwrap_or(PolicyEffect::Allow))
    }

    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<String>, PolicyEngineError> {
        let map = self.policies.read().map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
        Ok(map.get(bucket).cloned())
    }

    async fn put_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<(), PolicyEngineError> {
        // 验证 JSON 格式
        PolicyDocument::parse(policy_json)
            .map_err(|e| PolicyEngineError::InvalidPolicy(e.to_string()))?;
        let mut map = self.policies.write().map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
        map.insert(bucket.to_owned(), policy_json.to_owned());
        Ok(())
    }

    async fn delete_bucket_policy(&self, bucket: &str) -> Result<(), PolicyEngineError> {
        let mut map = self.policies.write().map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
        map.remove(bucket);
        Ok(())
    }
}
