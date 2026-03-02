use chrono::Utc;
use uuid::Uuid;

use crate::types::s3::core::*;

mod bucket;
mod config;
mod multipart;
mod object;

pub struct S3EngineImpl<S, M> {
    pub metadata: S,
    pub mount: M,
}

impl<S, M> S3EngineImpl<S, M> {
    pub fn new(metadata: S, mount: M) -> Self {
        Self { metadata, mount }
    }

    fn new_version_ref() -> ObjectVersionRef {
        ObjectVersionRef {
            version_id: Some(Uuid::new_v4().to_string()),
            is_latest: true,
            delete_marker: false,
        }
    }

    fn compute_etag(data: &[u8]) -> String {
        format!("{:x}", md5::compute(data))
    }

    fn now_doc(body: String) -> TimedDocument {
        TimedDocument { body, updated_at: Utc::now() }
    }
}
