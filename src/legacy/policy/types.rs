use serde::{Deserialize, Serialize};

// ── common ─────────────────────────────────────────────────────────

/// Effect of a policy: allow or deny.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    Allow,
    Deny,
}

impl Default for Effect {
    fn default() -> Self {
        Effect::Deny
    }
}

/// Visibility of a bucket/object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PrivilegeType {
    BucketList,
    BucketCreate,
    BucketDelete,
    BucketMetaGet,
    BucketMetaPut,
    ObjectGet,
    ObjectPut,
    ObjectDelete,
    ObjectList,
    ObjectMetaGet,
    ObjectMetaPut,
}

// ── privilege policy ───────────────────────────────────────────────

/// Controls what actions a user can perform.
///
/// ```json
/// {
///   "actions": ["bucket.list", "bucket.create"],
///   "effect": "allow",
///   "users": ["alice", "bob"]
/// }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PrivilegePolicy {
    /// Namespaced actions: "bucket.list", "bucket.create", "bucket.delete",
    /// "object.get", "object.put", "object.delete", "object.list".
    /// Use "bucket.*" or "object.*" for namespace wildcard, "*" for all.
    pub actions: Vec<String>,
    pub effect: Effect,
    /// Target users. Empty means all users.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<String>,
}

// ── resource policy ────────────────────────────────────────────────

/// Controls resource-level constraints (bucket, prefix, visibility).
///
/// ```json
/// {
///   "effect": "deny",
///   "users": ["guest"],
///   "bucket": "internal",
///   "prefix": "secret/",
///   "visibility": "private"
/// }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResourcePolicy {
    pub effect: Effect,
    /// Target users. Empty means all users.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<String>,
    /// Bucket name. Empty means all buckets.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bucket: String,
    /// Key prefix. Empty means all keys.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub prefix: String,
    /// Visibility constraint. None means any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
}

// ── policy statement (enum) ────────────────────────────────────────

/// A policy is either a privilege rule or a resource rule.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Policy {
    Privilege(PrivilegePolicy),
    Resource(ResourcePolicy),
}
