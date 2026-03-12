use chrono::SecondsFormat;
use thiserror::Error;
 use crate::types::traits::s3_policyengine::{PolicyEffect, PolicyEvalContext};
use crate::types::FS3Error;
use crate::types::s3::core::{
    BucketFeatures, CompleteMultipartInput, DeleteObjectOptions, ListOptions,
    ObjectWriteOptions, StorageClass, UploadedPart, VersioningState,
};
use crate::types::traits::StdError;
use crate::types::s3::policy::S3Action;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_policyengine::S3PolicyEngine;

#[derive(Debug, Error)]
pub enum S3HandlerBridgeError {
    #[error("{0}")]
    Unsupported(&'static str),
    #[error("{0}")]
    InvalidRequest(String),
    #[error("{0}")]
    AccessDenied(String),
    #[error("PreconditionFailed")]
    PreconditionFailed,
    #[error("NotModified")]
    NotModified,
    #[error("{0}")]
    InvalidVersioningStatus(String),
    #[error("{0}")]
    XmlParse(String),
}

impl From<S3HandlerBridgeError> for FS3Error {
    fn from(value: S3HandlerBridgeError) -> Self {
        match value {
            S3HandlerBridgeError::Unsupported(message) => FS3Error::bad_request(message),
            S3HandlerBridgeError::InvalidRequest(message) => FS3Error::bad_request(message),
            S3HandlerBridgeError::AccessDenied(message) => FS3Error::forbidden(message),
            S3HandlerBridgeError::PreconditionFailed => {
                FS3Error::precondition_failed("PreconditionFailed")
            }
            S3HandlerBridgeError::NotModified => FS3Error::not_modified("NotModified"),
            S3HandlerBridgeError::InvalidVersioningStatus(message) => {
                FS3Error::bad_request(message)
            }
            S3HandlerBridgeError::XmlParse(message) => FS3Error::bad_request(message),
        }
    }
}

pub async fn check_access<P, E>(
    policy: &P,
    action: S3Action,
    bucket: Option<&str>,
    key: Option<&str>,
) -> Result<(), E>
where
    P: S3PolicyEngine<E> + ?Sized,
    E: StdError + From<FS3Error>,
{
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
        Ok(PolicyEffect::Deny) => Err(FS3Error::forbidden(format!("{action}")).into()),
        Err(err) => Err(err),
    }
}

pub fn unsupported<T>(name: &'static str) -> Result<T, FS3Error> {
    Err(S3HandlerBridgeError::Unsupported(name).into())
}

pub fn to_resp_object(v: &crate::types::s3::core::S3Object) -> ObjectInfo {
    ObjectInfo {
        bucket: v.bucket.clone(),
        key: v.key.clone(),
        size: v.size,
        etag: Some(v.etag.clone()),
        last_modified: Some(v.last_modified.to_rfc3339_opts(SecondsFormat::Secs, true)),
        storage_class: Some(format!("{:?}", v.storage_class)),
        user_defined: v.user_metadata.clone(),
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

pub fn to_write_opt(content_type: Option<String>, size: u64, user_metadata: std::collections::HashMap<String, String>) -> ObjectWriteOptions {
    ObjectWriteOptions {
        content_type,
        content_encoding: None,
        storage_class: StorageClass::Standard,
        user_metadata,
        user_tags: std::collections::HashMap::new(),
        checksum: None,
        versioning: VersioningState::Off,
        retention: None,
        legal_hold: None,
        sse_algorithm: None,
        size,
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


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopySourceRef {
    pub bucket: String,
    pub key: String,
    pub version_id: Option<String>,
}

pub fn parse_copy_source(s: &str) -> Option<CopySourceRef> {
    let trimmed = s.trim();
    let raw = trimmed.strip_prefix('/').unwrap_or(trimmed);
    let (path_part, query_part) = match raw.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (raw, None),
    };
    let mut it = path_part.splitn(2, '/');
    let bucket = it.next()?.to_string();
    let key = it.next()?.to_string();
    let version_id = query_part.and_then(|query| {
        query.split('&').find_map(|pair| {
            let (name, value) = pair.split_once('=')?;
            if name == "versionId" {
                Some(value.to_string())
            } else {
                None
            }
        })
    });
    Some(CopySourceRef {
        bucket,
        key,
        version_id,
    })
}

pub fn copy_to_write_opt(req: &CopyObjectRequest) -> ObjectWriteOptions {
    let replace_metadata = req
        .metadata_directive
        .as_deref()
        .map(|value| value.eq_ignore_ascii_case("REPLACE"))
        .unwrap_or(false);
    let storage_class = match req.storage_class.as_deref() {
        Some("STANDARD_IA") => StorageClass::StandardIa,
        Some("ONEZONE_IA") => StorageClass::OneZoneIa,
        Some("GLACIER") => StorageClass::Glacier,
        Some("DEEP_ARCHIVE") => StorageClass::DeepArchive,
        Some("INTELLIGENT_TIERING") => StorageClass::IntelligentTiering,
        Some("REDUCED_REDUNDANCY") => StorageClass::ReducedRedundancy,
        Some(other) => StorageClass::Custom(other.to_string()),
        None => StorageClass::Standard,
    };

    ObjectWriteOptions {
        content_type: if replace_metadata {
            req.content_type.clone()
        } else {
            None
        },
        content_encoding: if replace_metadata {
            req.content_encoding.clone()
        } else {
            None
        },
        storage_class,
        user_metadata: if replace_metadata {
            req.user_metadata.clone()
        } else {
            std::collections::HashMap::new()
        },
        user_tags: std::collections::HashMap::new(),
        checksum: None,
        versioning: VersioningState::Off,
        retention: None,
        legal_hold: None,
        sse_algorithm: None,
        size: 0,
        copy_source_version_id: req.copy_source_version_id.clone(),
        metadata_directive: req.metadata_directive.clone(),
        tagging_directive: req.tagging_directive.clone(),
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
    Some(src[s..e].trim().trim_matches('"').trim_matches('\'').to_string())
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

#[cfg(test)]
mod tests {
    use super::{copy_to_write_opt, parse_copy_source};
    use crate::types::s3::request::{CopyObjectRequest, ObjectRef};
    use std::collections::HashMap;

    #[test]
    fn parse_copy_source_extracts_version_id() {
        let parsed = parse_copy_source("/src-bucket/path/to.txt?versionId=v123&ignored=1")
            .expect("copy source should parse");
        assert_eq!(parsed.bucket, "src-bucket");
        assert_eq!(parsed.key, "path/to.txt");
        assert_eq!(parsed.version_id.as_deref(), Some("v123"));
    }

    #[test]
    fn copy_to_write_opt_only_applies_replace_metadata() {
        let mut user_metadata = HashMap::new();
        user_metadata.insert("color".to_string(), "blue".to_string());
        let req = CopyObjectRequest {
            object: ObjectRef {
                bucket: "dst".to_string(),
                object: "obj".to_string(),
            },
            copy_source: "/src/key".to_string(),
            copy_source_version_id: Some("ver-1".to_string()),
            metadata_directive: Some("REPLACE".to_string()),
            tagging_directive: Some("COPY".to_string()),
            content_type: Some("text/plain".to_string()),
            content_encoding: Some("gzip".to_string()),
            storage_class: Some("STANDARD_IA".to_string()),
            user_metadata,
        };

        let opt = copy_to_write_opt(&req);
        assert_eq!(opt.copy_source_version_id.as_deref(), Some("ver-1"));
        assert_eq!(opt.content_type.as_deref(), Some("text/plain"));
        assert_eq!(opt.content_encoding.as_deref(), Some("gzip"));
        assert_eq!(opt.user_metadata.get("color").map(String::as_str), Some("blue"));
    }
}
