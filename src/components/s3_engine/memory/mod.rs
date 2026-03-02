use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::s3::core::*;
use crate::types::traits::s3_handler::S3HandlerBridgeError;

mod bucket;
mod config;
mod multipart;
mod object;
mod state;

pub use state::MemoryState;

#[derive(Debug, thiserror::Error)]
pub enum MemoryS3EngineError {
    #[error("bucket not found: {0}")]
    BucketNotFound(String),
    #[error("bucket already exists: {0}")]
    BucketAlreadyExists(String),
    #[error("bucket is not empty: {0}")]
    BucketNotEmpty(String),
    #[error("object not found: {bucket}/{key}")]
    ObjectNotFound { bucket: String, key: String },
    #[error("object version not found: {bucket}/{key}:{version_id}")]
    ObjectVersionNotFound {
        bucket: String,
        key: String,
        version_id: String,
    },
    #[error("multipart upload not found: {0}")]
    MultipartNotFound(String),
    #[error("multipart part missing: upload_id={upload_id} part_number={part_number}")]
    MultipartPartMissing { upload_id: String, part_number: u32 },
    #[error("invalid range header: {0}")]
    InvalidRange(String),
    #[error("{0}")]
    HandlerBridge(#[from] S3HandlerBridgeError),
}

#[derive(Debug, Clone, Default)]
pub struct MemoryS3Engine {
    pub(super) state: Arc<RwLock<MemoryState>>,
}

impl MemoryS3Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub(super) fn new_version_ref() -> ObjectVersionRef {
        ObjectVersionRef {
            version_id: Some(Uuid::new_v4().to_string()),
            is_latest: true,
            delete_marker: false,
        }
    }

    pub(super) fn compute_etag(data: &[u8]) -> String {
        format!("{:x}", md5::compute(data))
    }

    pub(super) fn now_doc(body: String) -> TimedDocument {
        TimedDocument {
            body,
            updated_at: Utc::now(),
        }
    }

    pub(super) fn apply_range(body: &[u8], range: &str) -> Result<bytes::Bytes, MemoryS3EngineError> {
        let raw = range.trim();
        let raw = raw
            .strip_prefix("bytes=")
            .ok_or_else(|| MemoryS3EngineError::InvalidRange(raw.to_owned()))?;
        let (start_s, end_s) = raw
            .split_once('-')
            .ok_or_else(|| MemoryS3EngineError::InvalidRange(range.to_owned()))?;
        let len = body.len() as i64;

        let (start, end) = if start_s.is_empty() {
            let suffix: i64 = end_s
                .parse()
                .map_err(|_| MemoryS3EngineError::InvalidRange(range.to_owned()))?;
            let start = (len - suffix).max(0);
            (start, len.saturating_sub(1))
        } else {
            let start: i64 = start_s
                .parse()
                .map_err(|_| MemoryS3EngineError::InvalidRange(range.to_owned()))?;
            let end: i64 = if end_s.is_empty() {
                len.saturating_sub(1)
            } else {
                end_s
                    .parse()
                    .map_err(|_| MemoryS3EngineError::InvalidRange(range.to_owned()))?
            };
            (start, end.min(len.saturating_sub(1)))
        };

        if start < 0 || end < start || start >= len {
            return Err(MemoryS3EngineError::InvalidRange(range.to_owned()));
        }
        Ok(bytes::Bytes::copy_from_slice(&body[start as usize..=end as usize]))
    }
}
