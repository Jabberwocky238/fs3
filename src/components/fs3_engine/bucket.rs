use async_trait::async_trait;
use crate::types::traits::s3_engine::*;
use crate::types::s3::core::*;
use crate::types::errors::{S3EngineError, S3Error};
use super::FS3Engine;

fn map_s3_error(e: S3Error) -> S3EngineError {
    match e {
        S3Error::NoSuchBucket(msg) => S3EngineError::BucketNotFound(msg),
        S3Error::NoSuchKey(msg) => S3EngineError::Storage(msg),
        S3Error::Storage(e) => S3EngineError::Storage(e.to_string()),
    }
}

#[async_trait]
impl S3BucketEngine for FS3Engine {
    async fn make_bucket(&self, bucket: &str, _region: Option<&str>, _features: BucketFeatures) -> Result<S3Bucket, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        self.object_layer.make_bucket(&ctx, bucket, Default::default()).await
            .map_err(map_s3_error)?;
        Ok(S3Bucket {
            identity: BucketIdentity {
                name: bucket.to_string(),
                created_at: chrono::Utc::now(),
                deleted_at: None,
            },
            region: None,
            features: Default::default(),
            tags: Default::default(),
        })
    }

    async fn head_bucket(&self, bucket: &str) -> Result<S3Bucket, S3EngineError> {
        self.get_bucket(bucket).await
    }

    async fn get_bucket(&self, bucket: &str) -> Result<S3Bucket, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let info = self.object_layer.get_bucket_info(&ctx, bucket, Default::default()).await
            .map_err(map_s3_error)?;
        Ok(S3Bucket {
            identity: BucketIdentity {
                name: info.name,
                created_at: chrono::DateTime::from_timestamp(info.created, 0).unwrap_or_default(),
                deleted_at: None,
            },
            region: None,
            features: Default::default(),
            tags: Default::default(),
        })
    }

    async fn list_buckets(&self) -> Result<Vec<S3Bucket>, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let buckets = self.object_layer.list_buckets(&ctx, Default::default()).await
            .map_err(map_s3_error)?;
        Ok(buckets.into_iter().map(|b| S3Bucket {
            identity: BucketIdentity {
                name: b.name,
                created_at: chrono::DateTime::from_timestamp(b.created, 0).unwrap_or_default(),
                deleted_at: None,
            },
            region: None,
            features: Default::default(),
            tags: Default::default(),
        }).collect())
    }

    async fn delete_bucket(&self, bucket: &str, force: bool) -> Result<(), S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let opts = crate::types::s3::object_layer_types::DeleteBucketOptions { force };
        self.object_layer.delete_bucket(&ctx, bucket, opts).await
            .map_err(map_s3_error)
    }

    async fn list_objects_v1(&self, _bucket: &str, _options: ListOptions) -> Result<ObjectListPage, S3EngineError> {
        Ok(ObjectListPage::default())
    }

    async fn list_objects_v2(&self, bucket: &str, options: ListOptions) -> Result<ObjectListPage, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };

        // Get storage path - need to access XlStorage's path
        let storage_path = std::path::PathBuf::from(".debug/fs3"); // TODO: get from config
        let bucket_path = storage_path.join(bucket);
        let mut objects = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&bucket_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let object_name = path.file_name().unwrap().to_string_lossy().to_string();
                    if object_name.starts_with('.') {
                        continue;
                    }
                    if let Some(ref prefix) = options.prefix {
                        if !object_name.starts_with(prefix) {
                            continue;
                        }
                    }
                    if let Ok(info) = self.object_layer.get_object_info(&ctx, bucket, &object_name, Default::default()).await {
                        objects.push(crate::types::s3::core::S3Object {
                            bucket: bucket.to_string(),
                            key: object_name,
                            size: info.size as u64,
                            etag: info.etag,
                            last_modified: chrono::Utc::now(),
                            content_type: Some(info.content_type),
                            content_encoding: None,
                            storage_class: Default::default(),
                            user_metadata: Default::default(),
                            user_tags: Default::default(),
                            version: Default::default(),
                            parts: Vec::new(),
                            checksums: Vec::new(),
                            replication_state: Default::default(),
                            retention: None,
                            legal_hold: None,
                            restore_expiry: None,
                            restore_ongoing: false,
                        });
                    }
                }
            }
        }

        let max_keys = options.max_keys.unwrap_or(1000) as usize;
        let is_truncated = objects.len() > max_keys;
        if is_truncated {
            objects.truncate(max_keys);
        }

        Ok(ObjectListPage {
            objects,
            common_prefixes: Vec::new(),
            next_continuation_token: None,
            next_key_marker: None,
            next_version_id_marker: None,
            is_truncated,
        })
    }

    async fn list_object_versions(&self, _bucket: &str, _options: ListOptions) -> Result<ObjectListPage, S3EngineError> {
        Ok(ObjectListPage::default())
    }
}
