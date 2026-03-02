use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;

use crate::types::traits::s3_metadata_storage::S3MetadataStoragePolicy;
use crate::types::traits::s3_policyengine::*;
use super::policy_doc::PolicyDocument;

/// 桶策略引擎：内存缓存 + storage 持久化
pub struct FS3BucketPolicyEngine<S> {
    storage: S,
    cache: RwLock<HashMap<String, String>>,
}

impl<S: S3MetadataStoragePolicy> FS3BucketPolicyEngine<S> {
    pub async fn new(storage: S) -> Result<Self, PolicyEngineError> {
        Ok(Self {
            storage,
            cache: RwLock::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl<S: S3MetadataStoragePolicy + Send + Sync> S3BucketPolicyEngine for FS3BucketPolicyEngine<S> {
    async fn is_allowed(&self, bucket: &str, ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        // 先查缓存
        let json = {
            let c = self.cache.read().map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
            c.get(bucket).cloned()
        };
        let json = match json {
            Some(j) => j,
            None => {
                // 缓存未命中，从 storage 加载
                let loaded = self.storage.load_bucket_policy(bucket).await
                    .map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
                match loaded {
                    Some(j) => {
                        let mut c = self.cache.write().map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
                        c.insert(bucket.to_owned(), j.clone());
                        j
                    }
                    None => return Ok(PolicyEffect::Allow),
                }
            }
        };
        let doc = PolicyDocument::parse(&json)
            .map_err(|e| PolicyEngineError::InvalidPolicy(e.to_string()))?;
        Ok(doc.evaluate(ctx).unwrap_or(PolicyEffect::Allow))
    }

    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<String>, PolicyEngineError> {
        self.storage.load_bucket_policy(bucket).await
            .map_err(|e| PolicyEngineError::Storage(e.to_string()))
    }

    async fn put_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<(), PolicyEngineError> {
        PolicyDocument::parse(policy_json)
            .map_err(|e| PolicyEngineError::InvalidPolicy(e.to_string()))?;
        self.storage.store_bucket_policy(bucket, policy_json).await
            .map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
        let mut c = self.cache.write().map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
        c.insert(bucket.to_owned(), policy_json.to_owned());
        Ok(())
    }

    async fn delete_bucket_policy(&self, bucket: &str) -> Result<(), PolicyEngineError> {
        self.storage.delete_bucket_policy(bucket).await
            .map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
        let mut c = self.cache.write().map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
        c.remove(bucket);
        Ok(())
    }
}
