use async_trait::async_trait;

use crate::types::traits::s3_policyengine::*;

/// 默认 IAM 策略引擎：无 IAM 策略时全部允许
pub struct FS3IamPolicyEngine;

#[async_trait]
impl S3IamPolicyEngine for FS3IamPolicyEngine {
    async fn is_allowed(&self, _ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        // 无 IAM 策略配置时默认允许
        Ok(PolicyEffect::Allow)
    }

    async fn get_user_policies(&self, _identity: &str) -> Result<Vec<String>, PolicyEngineError> {
        Ok(Vec::new())
    }

    async fn get_group_policies(&self, _group: &str) -> Result<Vec<String>, PolicyEngineError> {
        Ok(Vec::new())
    }
}
