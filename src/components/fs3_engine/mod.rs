use chrono::Utc;
use uuid::Uuid;

use crate::types::s3::core::*;

mod bucket;
mod config;
mod multipart;
mod object;

pub struct FS3Engine<S, M> {
    pub metadata: S,
    pub mount: M,
}

impl<S, M> FS3Engine<S, M> {
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

    /// Compute multipart ETag: MD5(md5_part1 || md5_part2 || ... || md5_partN)-N
    fn compute_multipart_etag(part_etags: &[String]) -> String {
        let mut bin = Vec::new();
        for etag_hex in part_etags {
            if let Ok(bytes) = hex::decode(etag_hex) {
                bin.extend_from_slice(&bytes);
            }
        }
        format!("{:x}-{}", md5::compute(&bin), part_etags.len())
    }

    fn now_doc(body: String) -> TimedDocument {
        TimedDocument { body, updated_at: Utc::now() }
    }
}
