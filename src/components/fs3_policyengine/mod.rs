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
