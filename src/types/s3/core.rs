use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type UserMetadata = HashMap<String, String>;
pub type TagMap = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VersioningState {
    Off,
    Enabled,
    Suspended,
}

impl Default for VersioningState {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObjectLockMode {
    Governance,
    Compliance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReplicationState {
    None,
    Pending,
    Completed,
    Failed,
    Replica,
}

impl Default for ReplicationState {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StorageClass {
    Standard,
    StandardIa,
    OneZoneIa,
    Glacier,
    DeepArchive,
    IntelligentTiering,
    ReducedRedundancy,
    Custom(String),
}

impl Default for StorageClass {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BucketFeatures {
    pub versioning: VersioningState,
    pub object_lock_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BucketIdentity {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct S3Bucket {
    pub identity: BucketIdentity,
    pub region: Option<String>,
    pub features: BucketFeatures,
    pub tags: TagMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectChecksum {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ObjectPart {
    pub part_number: u32,
    pub etag: String,
    pub size: u64,
    pub checksum: Option<ObjectChecksum>,
    pub last_modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ObjectVersionRef {
    pub version_id: Option<String>,
    pub is_latest: bool,
    pub delete_marker: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectRetention {
    pub mode: ObjectLockMode,
    pub retain_until: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ObjectLegalHold {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct S3Object {
    pub bucket: String,
    pub key: String,
    pub size: u64,
    pub etag: String,
    pub last_modified: DateTime<Utc>,
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub storage_class: StorageClass,
    pub user_metadata: UserMetadata,
    pub user_tags: TagMap,
    pub version: ObjectVersionRef,
    pub parts: Vec<ObjectPart>,
    pub checksums: Vec<ObjectChecksum>,
    pub replication_state: ReplicationState,
    pub retention: Option<ObjectRetention>,
    pub legal_hold: Option<ObjectLegalHold>,
    pub restore_expiry: Option<DateTime<Utc>>,
    pub restore_ongoing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ListOptions {
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<u32>,
    pub continuation_token: Option<String>,
    pub start_after: Option<String>,
    pub marker: Option<String>,
    pub key_marker: Option<String>,
    pub version_id_marker: Option<String>,
    pub fetch_owner: bool,
    pub include_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ObjectListPage {
    pub objects: Vec<S3Object>,
    pub common_prefixes: Vec<String>,
    pub next_continuation_token: Option<String>,
    pub next_key_marker: Option<String>,
    pub next_version_id_marker: Option<String>,
    pub is_truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultipartUpload {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub initiated_at: DateTime<Utc>,
    pub storage_class: StorageClass,
    pub user_metadata: UserMetadata,
    pub user_tags: TagMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UploadedPart {
    pub part_number: u32,
    pub etag: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CompleteMultipartInput {
    pub parts: Vec<UploadedPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ObjectReadOptions {
    pub version_id: Option<String>,
    pub range: Option<String>,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ObjectWriteOptions {
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub storage_class: StorageClass,
    pub user_metadata: UserMetadata,
    pub user_tags: TagMap,
    pub checksum: Option<ObjectChecksum>,
    pub versioning: VersioningState,
    pub retention: Option<ObjectRetention>,
    pub legal_hold: Option<ObjectLegalHold>,
    pub sse_algorithm: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteObjectOptions {
    pub version_id: Option<String>,
    pub bypass_governance: bool,
    pub replication_request: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct S3ErrorDetail {
    pub code: String,
    pub message: String,
    pub key: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeleteResult {
    pub deleted: Vec<ObjectVersionRef>,
    pub errors: Vec<S3ErrorDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BucketPolicyStatus {
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimedDocument {
    pub body: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BucketMetadataBundle {
    pub policy_json: Option<TimedDocument>,
    pub notification_xml: Option<TimedDocument>,
    pub lifecycle_xml: Option<TimedDocument>,
    pub object_lock_xml: Option<TimedDocument>,
    pub versioning_xml: Option<TimedDocument>,
    pub encryption_xml: Option<TimedDocument>,
    pub tagging_xml: Option<TimedDocument>,
    pub quota_json: Option<TimedDocument>,
    pub replication_xml: Option<TimedDocument>,
    pub targets_json: Option<TimedDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ReplicationMetrics {
    pub raw_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ReplicationCredsValidation {
    pub valid: bool,
    pub detail: Option<String>,
}
