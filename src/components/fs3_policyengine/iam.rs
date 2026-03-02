use async_trait::async_trait;

use crate::types::traits::s3_metadata_storage::S3MetadataStoragePolicy;
use crate::types::traits::s3_policyengine::*;
use super::policy_doc::PolicyDocument;

/// IAM 策略引擎：从 storage 读取用户/组策略并评估
pub struct FS3IamPolicyEngine<S> {
    storage: S,
}

impl<S> FS3IamPolicyEngine<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<S: S3MetadataStoragePolicy + Send + Sync> S3IamPolicyEngine for FS3IamPolicyEngine<S> {
    async fn is_allowed(&self, ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        // 收集用户直接绑定的策略
        let user_policies = self.get_user_policies(&ctx.identity).await?;
        // 收集用户所属组的策略
        let mut all_policies = user_policies;
        for group in &ctx.groups {
            let gp = self.get_group_policies(group).await?;
            all_policies.extend(gp);
        }

        if all_policies.is_empty() {
            return Ok(PolicyEffect::Allow); // 无 IAM 策略时默认允许
        }

        let mut has_allow = false;
        for name in &all_policies {
            let json = self.storage.load_iam_policy(name).await
                .map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
            let json = match json {
                Some(j) => j,
                None => continue,
            };
            let doc = PolicyDocument::parse(&json)
                .map_err(|e| PolicyEngineError::InvalidPolicy(e.to_string()))?;
            match doc.evaluate(ctx) {
                Some(PolicyEffect::Deny) => return Ok(PolicyEffect::Deny),
                Some(PolicyEffect::Allow) => has_allow = true,
                None => {}
            }
        }
        Ok(if has_allow { PolicyEffect::Allow } else { PolicyEffect::Deny })
    }

    async fn get_user_policies(&self, identity: &str) -> Result<Vec<String>, PolicyEngineError> {
        let mapping = self.storage.load_user_policy_mapping(identity).await
            .map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
        Ok(parse_csv_names(mapping.as_deref()))
    }

    async fn get_group_policies(&self, group: &str) -> Result<Vec<String>, PolicyEngineError> {
        let mapping = self.storage.load_group_policy_mapping(group).await
            .map_err(|e| PolicyEngineError::Storage(e.to_string()))?;
        Ok(parse_csv_names(mapping.as_deref()))
    }
}

fn parse_csv_names(s: Option<&str>) -> Vec<String> {
    match s {
        None => Vec::new(),
        Some(s) => s.split(',').map(|n| n.trim().to_owned()).filter(|n| !n.is_empty()).collect(),
    }
}
