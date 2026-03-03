mod bucket_policy;
mod iam;
mod policy_doc;

use async_trait::async_trait;

use crate::types::traits::s3_policyengine::*;

/// 组合策略引擎：串联 IAM + 桶策略
pub struct FS3PolicyEngine<I, B> {
    pub iam: I,
    pub bucket: B,
}

impl<I, B> FS3PolicyEngine<I, B> {
    pub fn new(iam: I, bucket: B) -> Self {
        Self { iam, bucket }
    }
}

#[async_trait]
impl<I, B> S3IamPolicyEngine for FS3PolicyEngine<I, B>
where
    I: S3IamPolicyEngine + Send + Sync,
    B: S3BucketPolicyEngine + Send + Sync,
{
    async fn is_allowed(&self, ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        self.iam.is_allowed(ctx).await
    }

    async fn get_user_policies(&self, identity: &str) -> Result<Vec<String>, PolicyEngineError> {
        self.iam.get_user_policies(identity).await
    }

    async fn get_group_policies(&self, group: &str) -> Result<Vec<String>, PolicyEngineError> {
        self.iam.get_group_policies(group).await
    }
}

#[async_trait]
impl<I, B> S3BucketPolicyEngine for FS3PolicyEngine<I, B>
where
    I: S3IamPolicyEngine + Send + Sync,
    B: S3BucketPolicyEngine + Send + Sync,
{
    async fn is_allowed(&self, bucket: &str, ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        self.bucket.is_allowed(bucket, ctx).await
    }

    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<String>, PolicyEngineError> {
        self.bucket.get_bucket_policy(bucket).await
    }

    async fn put_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<(), PolicyEngineError> {
        self.bucket.put_bucket_policy(bucket, policy_json).await
    }

    async fn delete_bucket_policy(&self, bucket: &str) -> Result<(), PolicyEngineError> {
        self.bucket.delete_bucket_policy(bucket).await
    }
}

#[async_trait]
impl<I, B> S3PolicyEngine for FS3PolicyEngine<I, B>
where
    I: S3IamPolicyEngine + Send + Sync,
    B: S3BucketPolicyEngine + Send + Sync,
{
    async fn check_access(&self, ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        // owner（root 凭证）直接放行
        if ctx.is_owner {
            return Ok(PolicyEffect::Allow);
        }

        // IAM 显式拒绝优先
        let iam_result = self.iam.is_allowed(ctx).await?;
        if iam_result == PolicyEffect::Deny {
            return Ok(PolicyEffect::Deny);
        }

        // 桶策略评估
        if let Some(bucket) = &ctx.bucket {
            let bp_result = self.bucket.is_allowed(bucket, ctx).await?;
            if bp_result == PolicyEffect::Deny {
                return Ok(PolicyEffect::Deny);
            }
            // 桶策略显式允许可以授权访问
            if bp_result == PolicyEffect::Allow {
                return Ok(PolicyEffect::Allow);
            }
        }

        // IAM 允许即放行
        Ok(iam_result)
    }
}

/// 默认策略引擎：owner 全部放行，非 owner 全部拒绝
pub struct DefaultPolicyEngine;

#[async_trait]
impl S3IamPolicyEngine for DefaultPolicyEngine {
    async fn is_allowed(&self, _ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        Ok(PolicyEffect::Deny)
    }
    async fn get_user_policies(&self, _identity: &str) -> Result<Vec<String>, PolicyEngineError> {
        Ok(Vec::new())
    }
    async fn get_group_policies(&self, _group: &str) -> Result<Vec<String>, PolicyEngineError> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl S3BucketPolicyEngine for DefaultPolicyEngine {
    async fn is_allowed(&self, _bucket: &str, _ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        Ok(PolicyEffect::Deny)
    }
    async fn get_bucket_policy(&self, _bucket: &str) -> Result<Option<String>, PolicyEngineError> {
        Ok(None)
    }
    async fn put_bucket_policy(&self, _bucket: &str, _policy_json: &str) -> Result<(), PolicyEngineError> {
        Ok(())
    }
    async fn delete_bucket_policy(&self, _bucket: &str) -> Result<(), PolicyEngineError> {
        Ok(())
    }
}

#[async_trait]
impl S3PolicyEngine for DefaultPolicyEngine {
    async fn check_access(&self, ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError> {
        if ctx.is_owner {
            Ok(PolicyEffect::Allow)
        } else {
            Ok(PolicyEffect::Deny)
        }
    }
}

pub use bucket_policy::FS3BucketPolicyEngine;
pub use iam::FS3IamPolicyEngine;
pub use policy_doc::PolicyDocument;
