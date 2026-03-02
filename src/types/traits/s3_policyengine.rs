use async_trait::async_trait;

/// S3 策略引擎的请求上下文，包含评估策略所需的全部信息
#[derive(Debug, Clone)]
pub struct PolicyEvalContext {
    /// S3 动作，如 "s3:GetObject", "s3:PutObject"
    pub action: String,
    /// 目标桶名
    pub bucket: Option<String>,
    /// 目标对象 key
    pub key: Option<String>,
    /// 请求者身份标识（access key 或用户名）
    pub identity: String,
    /// 请求者所属组
    pub groups: Vec<String>,
    /// 是否为 owner（root 凭证）
    pub is_owner: bool,
    /// 请求条件键值对（如 source IP、referer 等）
    pub conditions: std::collections::HashMap<String, String>,
}

/// 策略评估结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

/// 策略引擎错误
#[derive(Debug, thiserror::Error)]
pub enum PolicyEngineError {
    #[error("policy not found: {0}")]
    PolicyNotFound(String),
    #[error("invalid policy document: {0}")]
    InvalidPolicy(String),
    #[error("storage error: {0}")]
    Storage(String),
}

/// IAM 级策略引擎 — 身份级访问控制
#[async_trait]
pub trait S3IamPolicyEngine: Send + Sync {
    /// 评估 IAM 策略（用户策略 + 组策略合并）
    async fn is_allowed(&self, ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError>;

    /// 获取用户关联的策略名列表
    async fn get_user_policies(&self, identity: &str) -> Result<Vec<String>, PolicyEngineError>;

    /// 获取组关联的策略名列表
    async fn get_group_policies(&self, group: &str) -> Result<Vec<String>, PolicyEngineError>;
}

/// 桶级策略引擎 — 资源级访问控制
#[async_trait]
pub trait S3BucketPolicyEngine: Send + Sync {
    /// 评估桶策略
    async fn is_allowed(&self, bucket: &str, ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError>;

    /// 获取桶策略文档（JSON）
    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<String>, PolicyEngineError>;

    /// 设置桶策略文档
    async fn put_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<(), PolicyEngineError>;

    /// 删除桶策略
    async fn delete_bucket_policy(&self, bucket: &str) -> Result<(), PolicyEngineError>;
}

/// 组合策略引擎 — 统一入口，串联 IAM + 桶策略
#[async_trait]
pub trait S3PolicyEngine: Send + Sync {
    /// 综合评估：owner 直接放行，否则 IAM 策略 + 桶策略联合判定
    async fn check_access(&self, ctx: &PolicyEvalContext) -> Result<PolicyEffect, PolicyEngineError>;
}
