use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Context {
    pub request_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct ObjectOptions {
    pub version_id: Option<String>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub content_md5: Option<String>,
    pub checksum: Option<String>,
    pub user_defined: HashMap<String, String>,
    pub range: Option<(u64, u64)>,
}

#[derive(Debug, Clone, Default)]
pub struct MakeBucketOptions {
    pub lock_enabled: bool,
    pub versioning_enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BucketOptions {
    pub deleted: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DeleteBucketOptions {
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketInfo {
    pub name: String,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    pub bucket: String,
    pub name: String,
    pub size: u64,
    pub etag: String,
    pub content_type: String,
    pub user_defined: HashMap<String, String>,
}
