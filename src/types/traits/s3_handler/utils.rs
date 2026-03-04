use chrono::SecondsFormat;
use thiserror::Error;

use crate::types::s3::core::{
    BucketFeatures, CompleteMultipartInput, DeleteObjectOptions, ListOptions,
    ObjectWriteOptions, StorageClass, UploadedPart, VersioningState,
};
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::s3::request::*;
use crate::types::s3::response::*;

#[derive(Debug, Error)]
pub enum S3HandlerBridgeError {
    #[error("unsupported by current S3 engine: {0}")]
    Unsupported(&'static str),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("access denied: {0}")]
    AccessDenied(String),
    #[error("precondition failed")]
    PreconditionFailed,
    #[error("not modified")]
    NotModified,
    #[error("invalid versioning status: {0}")]
    InvalidVersioningStatus(String),
    #[error("XML parse error: {0}")]
    XmlParse(String),
}

pub async fn check_access<P: S3PolicyEngine + ?Sized>(
    policy: &P,
    action: S3Action,
    bucket: Option<&str>,
    key: Option<&str>,
) -> Result<(), S3HandlerBridgeError> {
    use crate::types::traits::s3_policyengine::{PolicyEvalContext, PolicyEffect};
    let ctx = PolicyEvalContext {
        action,
        bucket: bucket.map(|s| s.to_string()),
        key: key.map(|s| s.to_string()),
        identity: String::new(),
        groups: Vec::new(),
        is_owner: false,
        conditions: std::collections::HashMap::new(),
    };
    match policy.check_access(&ctx).await {
        Ok(PolicyEffect::Allow) => Ok(()),
        Ok(PolicyEffect::Deny) => Err(S3HandlerBridgeError::AccessDenied(format!("{action}"))),
        Err(e) => Err(S3HandlerBridgeError::AccessDenied(e.to_string())),
    }
}

pub fn unsupported<T, E>(op: &'static str) -> Result<T, E>
where
    E: From<S3HandlerBridgeError>,
{
    Err(S3HandlerBridgeError::Unsupported(op).into())
}

pub fn to_resp_object(v: &crate::types::s3::core::S3Object) -> ObjectInfo {
    ObjectInfo {
        bucket: v.bucket.clone(),
        key: v.key.clone(),
        size: v.size,
        etag: Some(v.etag.clone()),
        last_modified: Some(v.last_modified.to_rfc3339_opts(SecondsFormat::Secs, true)),
        storage_class: Some(format!("{:?}", v.storage_class)),
    }
}

pub fn to_list_opt(query: &ListQuery, include_metadata: bool) -> ListOptions {
    ListOptions {
        prefix: query.prefix.clone(),
        delimiter: query.delimiter.clone(),
        max_keys: query.max_keys,
        continuation_token: query.continuation_token.clone(),
        start_after: query.start_after.clone(),
        marker: query.marker.clone(),
        key_marker: query.key_marker.clone(),
        version_id_marker: query.version_id_marker.clone(),
        fetch_owner: false,
        include_metadata,
    }
}

pub fn to_write_opt(content_type: Option<String>) -> ObjectWriteOptions {
    ObjectWriteOptions {
        content_type,
        content_encoding: None,
        storage_class: StorageClass::Standard,
        user_metadata: std::collections::HashMap::new(),
        user_tags: std::collections::HashMap::new(),
        checksum: None,
        versioning: VersioningState::Off,
        retention: None,
        legal_hold: None,
        sse_algorithm: None,
    }
}

pub fn to_delete_opt(version_id: Option<String>) -> DeleteObjectOptions {
    DeleteObjectOptions {
        version_id,
        bypass_governance: false,
        replication_request: false,
    }
}

pub fn bucket_features_for_create() -> BucketFeatures {
    BucketFeatures {
        versioning: VersioningState::Off,
        object_lock_enabled: false,
    }
}

pub fn split_copy_source(s: &str) -> Option<(String, String)> {
    let raw = s.trim_start_matches('/');
    let mut it = raw.splitn(2, '/');
    let b = it.next()?.to_string();
    let k = it.next()?.to_string();
    Some((b, k))
}

pub fn parse_completed_parts(xml: &str) -> CompleteMultipartInput {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(part_start) = xml[pos..].find("<Part>") {
        let ps = pos + part_start;
        let pe = match xml[ps..].find("</Part>") {
            Some(v) => ps + v,
            None => break,
        };
        let chunk = &xml[ps..pe];
        let pn = extract_tag(chunk, "PartNumber")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or_default();
        let etag = extract_tag(chunk, "ETag").unwrap_or_default();
        out.push(UploadedPart {
            part_number: pn,
            etag,
            size: 0,
        });
        pos = pe + "</Part>".len();
    }
    CompleteMultipartInput { parts: out }
}

pub fn extract_tag(src: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let s = src.find(&open)? + open.len();
    let e = src[s..].find(&close)? + s;
    Some(
        src[s..e]
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string(),
    )
}

pub fn parse_delete_keys(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(start) = xml[pos..].find("<Key>") {
        let s = pos + start + "<Key>".len();
        let e = match xml[s..].find("</Key>") {
            Some(v) => s + v,
            None => break,
        };
        out.push(xml[s..e].to_string());
        pos = e + "</Key>".len();
    }
    out
}

