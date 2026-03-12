use async_trait::async_trait;

use crate::types::s3::policy::S3Action;
use crate::types::traits::StdError;

#[derive(Debug, Clone)]
pub struct PolicyEvalContext {
    pub action: S3Action,
    pub bucket: Option<String>,
    pub key: Option<String>,
    pub identity: String,
    pub groups: Vec<String>,
    pub is_owner: bool,
    pub conditions: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Debug, thiserror::Error)]
pub enum PolicyEngineError {
    #[error("policy not found: {0}")]
    PolicyNotFound(String),
    #[error("invalid policy document: {0}")]
    InvalidPolicy(String),
    #[error("storage error: {0}")]
    Storage(String),
}

#[async_trait]
pub trait S3IamPolicyEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn is_allowed(&self, ctx: &PolicyEvalContext) -> Result<PolicyEffect, E>;
    async fn get_user_policies(&self, identity: &str) -> Result<Vec<String>, E>;
    async fn get_group_policies(&self, group: &str) -> Result<Vec<String>, E>;
}

#[async_trait]
pub trait S3BucketPolicyEngine<E>: Send + Sync
where
    E: StdError,
{
    async fn is_allowed(&self, bucket: &str, ctx: &PolicyEvalContext) -> Result<PolicyEffect, E>;
    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<String>, E>;
    async fn put_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<(), E>;
    async fn delete_bucket_policy(&self, bucket: &str) -> Result<(), E>;
}

#[async_trait]
pub trait S3PolicyEngine<E>: Send + Sync + S3IamPolicyEngine<E> + S3BucketPolicyEngine<E>
where
    E: StdError,
{
    async fn check_access(&self, ctx: &PolicyEvalContext) -> Result<PolicyEffect, E>;
}
