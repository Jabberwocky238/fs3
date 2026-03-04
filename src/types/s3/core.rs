use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::pin::Pin;

pub type UserMetadata = HashMap<String, String>;
pub type TagMap = HashMap<String, String>;
pub type BoxByteStream = Pin<Box<dyn futures::Stream<Item = Result<bytes::Bytes, io::Error>> + Send>>;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VersioningState {
    #[default]
    Off,
    Enabled,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObjectLockMode {
    Governance,
    Compliance,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReplicationState {
    #[default]
    None,
    Pending,
    Completed,
    Failed,
    Replica,
}


#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StorageClass {
    #[default]
    Standard,
    StandardIa,
    OneZoneIa,
    Glacier,
    DeepArchive,
    IntelligentTiering,
    ReducedRedundancy,
    Custom(String),
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
pub enum ObjectAttribute {
    #[default]
    ETag,
    Checksum,
    ObjectParts,
    StorageClass,
    ObjectSize,
    LastModified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ObjectReadOptions {
    pub version_id: Option<String>,
    pub range: Option<(u64, u64)>,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
    pub want_etag: bool,
    pub want_checksum: bool,
    pub want_object_parts: bool,
    pub want_storage_class: bool,
    pub want_object_size: bool,
    pub want_last_modified: bool,
}

impl From<crate::types::s3::request::GetObjectAttributesRequest> for ObjectReadOptions {
    fn from(req: crate::types::s3::request::GetObjectAttributesRequest) -> Self {
        let mut out = Self {
            version_id: None,
            range: None,
            if_match: None,
            if_none_match: None,
            want_etag: false,
            want_checksum: false,
            want_object_parts: false,
            want_storage_class: false,
            want_object_size: false,
            want_last_modified: false,
        };
        for a in req.attributes {
            match a {
                ObjectAttribute::ETag => out.want_etag = true,
                ObjectAttribute::Checksum => out.want_checksum = true,
                ObjectAttribute::ObjectParts => out.want_object_parts = true,
                ObjectAttribute::StorageClass => out.want_storage_class = true,
                ObjectAttribute::ObjectSize => out.want_object_size = true,
                ObjectAttribute::LastModified => out.want_last_modified = true,
            }
        }
        // S3 GetObjectAttributes without explicit list still implies basic object attrs.
        if !out.want_etag
            && !out.want_checksum
            && !out.want_object_parts
            && !out.want_storage_class
            && !out.want_object_size
            && !out.want_last_modified
        {
            out.want_etag = true;
            out.want_object_size = true;
            out.want_last_modified = true;
        }
        out
    }
}

impl From<&crate::types::s3::request::HeadObjectRequest> for ObjectReadOptions {
    fn from(req: &crate::types::s3::request::HeadObjectRequest) -> Self {
        fn parse_range(s: &str) -> Option<(u64, u64)> {
            let s = s.strip_prefix("bytes=")?;
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 2 { return None; }
            Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
        }
        Self {
            version_id: req.version_id.clone(),
            range: req.range.as_ref().and_then(|r| parse_range(r)),
            if_match: req.if_match.clone(),
            if_none_match: req.if_none_match.clone(),
            want_etag: false,
            want_checksum: false,
            want_object_parts: false,
            want_storage_class: false,
            want_object_size: false,
            want_last_modified: false,
        }
    }
}

impl From<&crate::types::s3::request::GetObjectRequest> for ObjectReadOptions {
    fn from(req: &crate::types::s3::request::GetObjectRequest) -> Self {
        fn parse_range(s: &str) -> Option<(u64, u64)> {
            let s = s.strip_prefix("bytes=")?;
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 2 { return None; }
            Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
        }
        Self {
            version_id: req.version_id.clone(),
            range: req.range.as_ref().and_then(|r| parse_range(r)),
            if_match: None,
            if_none_match: None,
            want_etag: false,
            want_checksum: false,
            want_object_parts: false,
            want_storage_class: false,
            want_object_size: false,
            want_last_modified: false,
        }
    }
}

impl From<&crate::types::s3::request::GetObjectLambdaRequest> for ObjectReadOptions {
    fn from(req: &crate::types::s3::request::GetObjectLambdaRequest) -> Self {
        fn parse_range(s: &str) -> Option<(u64, u64)> {
            let s = s.strip_prefix("bytes=")?;
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 2 { return None; }
            Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
        }
        Self {
            version_id: req.version_id.clone(),
            range: req.range.as_ref().and_then(|r| parse_range(r)),
            if_match: req.if_match.clone(),
            if_none_match: req.if_none_match.clone(),
            want_etag: false,
            want_checksum: false,
            want_object_parts: false,
            want_storage_class: false,
            want_object_size: false,
            want_last_modified: false,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ReplicationMetrics {
    pub raw_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ReplicationCredsValidation {
    pub valid: bool,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BucketMetadataBundle {
    pub lifecycle_rules: Vec<String>,
    pub encryption_algorithm: Option<String>,
    pub encryption_key_id: Option<String>,
    pub object_lock_enabled: Option<bool>,
    pub object_lock_mode: Option<String>,
    pub object_lock_days: Option<u32>,
    pub object_lock_years: Option<u32>,
    pub versioning_status: Option<String>,
    pub versioning_mfa_delete: Option<String>,
    pub notification_configs: Vec<String>,
    pub replication_role: Option<String>,
    pub replication_rules: Vec<String>,
    pub tagging_map: Option<HashMap<String, String>>,
    pub cors: Option<CorsConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CorsConfiguration {
    #[serde(rename = "CORSRule", default)]
    pub rules: Vec<CorsRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CorsRule {
    #[serde(rename = "AllowedOrigin", default)]
    pub allowed_origins: Vec<String>,
    #[serde(rename = "AllowedMethod", default)]
    pub allowed_methods: Vec<String>,
    #[serde(rename = "AllowedHeader", default)]
    pub allowed_headers: Vec<String>,
    #[serde(rename = "ExposeHeader", default)]
    pub expose_headers: Vec<String>,
    #[serde(rename = "MaxAgeSeconds", default)]
    pub max_age_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BucketEncryption {
    pub algorithm: String,
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BucketObjectLockConfig {
    pub enabled: bool,
    pub mode: Option<String>,
    pub days: Option<u32>,
    pub years: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BucketVersioning {
    pub status: String,
    pub mfa_delete: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BucketReplication {
    pub role: String,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BucketWebsite {
    pub index_document: String,
    pub error_document: Option<String>,
}

