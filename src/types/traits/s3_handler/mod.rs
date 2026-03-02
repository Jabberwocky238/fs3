use async_trait::async_trait;
use chrono::SecondsFormat;
use thiserror::Error;

mod bucket;
mod object;
mod utils;

use crate::types::s3::core::{
    BucketFeatures, CompleteMultipartInput, DeleteObjectOptions, ListOptions, ObjectReadOptions,
    ObjectWriteOptions, StorageClass, UploadedPart, VersioningState,
};
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{
    S3BucketConfigEngine, S3BucketEngine, S3MultipartEngine, S3ObjectEngine,
};

pub use bucket::BucketS3Handler;
pub use object::ObjectS3Handler;

#[derive(Debug, Error)]
pub enum S3HandlerBridgeError {
    #[error("unsupported by current S3 engine: {0}")]
    Unsupported(&'static str),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

fn unsupported<T, E>(op: &'static str) -> Result<T, E>
where
    E: From<S3HandlerBridgeError>,
{
    Err(S3HandlerBridgeError::Unsupported(op).into())
}

fn to_resp_object(v: &crate::types::s3::core::S3Object) -> ObjectInfo {
    ObjectInfo {
        bucket: v.bucket.clone(),
        key: v.key.clone(),
        size: v.size,
        etag: Some(v.etag.clone()),
        last_modified: Some(v.last_modified.to_rfc3339_opts(SecondsFormat::Secs, true)),
        storage_class: Some(format!("{:?}", v.storage_class)),
    }
}

fn to_list_opt(query: &ListQuery, include_metadata: bool) -> ListOptions {
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

fn to_write_opt(content_type: Option<String>) -> ObjectWriteOptions {
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

fn to_delete_opt(version_id: Option<String>) -> DeleteObjectOptions {
    DeleteObjectOptions {
        version_id,
        bypass_governance: false,
        replication_request: false,
    }
}

fn bucket_features_for_create() -> BucketFeatures {
    BucketFeatures {
        versioning: VersioningState::Off,
        object_lock_enabled: false,
    }
}

fn split_copy_source(s: &str) -> Option<(String, String)> {
    let raw = s.trim_start_matches('/');
    let mut it = raw.splitn(2, '/');
    let b = it.next()?.to_string();
    let k = it.next()?.to_string();
    Some((b, k))
}

fn parse_completed_parts(xml: &str) -> CompleteMultipartInput {
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

fn extract_tag(src: &str, name: &str) -> Option<String> {
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

fn parse_delete_keys(xml: &str) -> Vec<String> {
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


#[async_trait]
pub trait RootS3Handler
where
    <Self::Engine as S3BucketEngine>::Error: Into<Self::Error>,
    Self::Error: From<S3HandlerBridgeError>,
{
    type Engine: S3BucketEngine;
    type Error: Send + Sync + 'static;
    fn engine(&self) -> &Self::Engine;

    async fn root_listen_notification(
        &self,
        _req: RootListenNotificationRequest,
    ) -> Result<RootListenNotificationResponse, Self::Error> {
        unsupported("RootListenNotification")
    }

    async fn list_buckets(&self, _req: ListBucketsRequest) -> Result<ListBucketsResponse, Self::Error> {
        let list = self.engine().list_buckets().await.map_err(Into::into)?;
        Ok(ListBucketsResponse {
            buckets: list
                .into_iter()
                .map(|b| BucketInfo {
                    name: b.identity.name,
                    creation_date: Some(b.identity.created_at.to_rfc3339()),
                })
                .collect(),
            ..Default::default()
        })
    }

    async fn list_buckets_double_slash(
        &self,
        _req: ListBucketsDoubleSlashRequest,
    ) -> Result<ListBucketsDoubleSlashResponse, Self::Error> {
        let list = self.engine().list_buckets().await.map_err(Into::into)?;
        Ok(ListBucketsDoubleSlashResponse {
            buckets: list
                .into_iter()
                .map(|b| BucketInfo {
                    name: b.identity.name,
                    creation_date: Some(b.identity.created_at.to_rfc3339()),
                })
                .collect(),
            ..Default::default()
        })
    }
}

#[async_trait]
pub trait RejectedObjectS3Handler {
    type Error: Send + Sync + 'static;

    async fn rejected_object_torrent(
        &self,
        req: RejectedObjectTorrentRequest,
    ) -> Result<RejectedApiResponse, Self::Error> {
        Ok(RejectedApiResponse {
            error: ErrorBody {
                code: "NotImplemented".to_string(),
                message: "Object torrent API is not implemented".to_string(),
                resource: Some(format!("{}/{} {}", req.object.bucket, req.object.object, req.method)),
            },
            ..Default::default()
        })
    }
    async fn rejected_object_acl_delete(
        &self,
        req: RejectedObjectAclDeleteRequest,
    ) -> Result<RejectedApiResponse, Self::Error> {
        Ok(RejectedApiResponse {
            error: ErrorBody {
                code: "NotImplemented".to_string(),
                message: "Object ACL delete API is not implemented".to_string(),
                resource: Some(format!("{}/{}", req.object.bucket, req.object.object)),
            },
            ..Default::default()
        })
    }
}

#[async_trait]
pub trait RejectedBucketS3Handler {
    type Error: Send + Sync + 'static;

    async fn rejected_bucket_api(
        &self,
        req: RejectedBucketApiRequest,
    ) -> Result<RejectedApiResponse, Self::Error> {
        Ok(RejectedApiResponse {
            error: ErrorBody {
                code: "NotImplemented".to_string(),
                message: format!("Bucket API {} is not implemented", req.api),
                resource: Some(format!("{} {}", req.bucket.bucket, req.method)),
            },
            ..Default::default()
        })
    }
}
