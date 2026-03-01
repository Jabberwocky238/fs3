use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};

use crate::components::s3_engine::memory::MemoryS3EngineError;
use crate::types::s3::core::*;

#[derive(Debug, Default)]
pub struct MemoryState {
    pub buckets: HashMap<String, S3Bucket>,
    pub objects: HashMap<(String, String), Vec<StoredObjectVersion>>,
    pub multiparts: HashMap<String, MultipartUploadState>,
    pub bucket_metadata: HashMap<String, BucketMetadataBundle>,
}

#[derive(Debug, Clone)]
pub struct StoredObjectVersion {
    pub object: S3Object,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct MultipartUploadState {
    pub upload: MultipartUpload,
    pub parts: BTreeMap<u32, MultipartPartData>,
    pub write_options: ObjectWriteOptions,
}

#[derive(Debug, Clone)]
pub struct MultipartPartData {
    pub uploaded: UploadedPart,
    pub body: Vec<u8>,
}

impl MemoryState {
    pub fn ensure_bucket(&self, bucket: &str) -> Result<(), MemoryS3EngineError> {
        if self.buckets.contains_key(bucket) {
            Ok(())
        } else {
            Err(MemoryS3EngineError::BucketNotFound(bucket.to_owned()))
        }
    }

    pub fn get_versions(&self, bucket: &str, key: &str) -> Result<&Vec<StoredObjectVersion>, MemoryS3EngineError> {
        self.objects
            .get(&(bucket.to_owned(), key.to_owned()))
            .ok_or_else(|| MemoryS3EngineError::ObjectNotFound {
                bucket: bucket.to_owned(),
                key: key.to_owned(),
            })
    }

    pub fn get_versions_mut(
        &mut self,
        bucket: &str,
        key: &str,
    ) -> Result<&mut Vec<StoredObjectVersion>, MemoryS3EngineError> {
        self.objects
            .get_mut(&(bucket.to_owned(), key.to_owned()))
            .ok_or_else(|| MemoryS3EngineError::ObjectNotFound {
                bucket: bucket.to_owned(),
                key: key.to_owned(),
            })
    }

    pub fn select_version_index(versions: &[StoredObjectVersion], version_id: Option<&str>) -> Option<usize> {
        match version_id {
            Some(vid) => versions
                .iter()
                .position(|v| v.object.version.version_id.as_deref() == Some(vid)),
            None => versions.len().checked_sub(1),
        }
    }

    pub fn put_object(
        &mut self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        options: ObjectWriteOptions,
        etag: String,
        now: DateTime<Utc>,
        version: ObjectVersionRef,
    ) -> Result<S3Object, MemoryS3EngineError> {
        self.ensure_bucket(bucket)?;

        let versions = self
            .objects
            .entry((bucket.to_owned(), key.to_owned()))
            .or_default();

        if let Some(last) = versions.last_mut() {
            last.object.version.is_latest = false;
        }

        let obj = S3Object {
            bucket: bucket.to_owned(),
            key: key.to_owned(),
            size: body.len() as u64,
            etag,
            last_modified: now,
            content_type: options.content_type.clone(),
            content_encoding: options.content_encoding.clone(),
            storage_class: options.storage_class.clone(),
            user_metadata: options.user_metadata.clone(),
            user_tags: options.user_tags.clone(),
            version,
            parts: vec![],
            checksums: options.checksum.into_iter().collect(),
            replication_state: ReplicationState::None,
            retention: options.retention.clone(),
            legal_hold: options.legal_hold.clone(),
            restore_expiry: None,
            restore_ongoing: false,
        };

        versions.push(StoredObjectVersion {
            object: obj.clone(),
            body,
        });
        Ok(obj)
    }
}
