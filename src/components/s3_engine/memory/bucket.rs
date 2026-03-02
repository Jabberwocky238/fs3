use std::collections::BTreeSet;

use async_trait::async_trait;

use crate::components::s3_engine::memory::{MemoryS3Engine, MemoryS3EngineError};
use crate::types::s3::core::*;
use crate::types::traits::s3_engine::S3BucketEngine;

#[async_trait]
impl S3BucketEngine<MemoryS3EngineError> for MemoryS3Engine {

    async fn make_bucket(
        &self,
        bucket: &str,
        region: Option<&str>,
        features: BucketFeatures,
    ) -> Result<S3Bucket, Self::Error> {
        let mut state = self.state.write().await;
        if state.buckets.contains_key(bucket) {
            return Err(MemoryS3EngineError::BucketAlreadyExists(bucket.to_owned()));
        }

        let bucket_obj = S3Bucket {
            identity: BucketIdentity {
                name: bucket.to_owned(),
                created_at: chrono::Utc::now(),
                deleted_at: None,
            },
            region: region.map(str::to_owned),
            features,
            tags: TagMap::new(),
        };
        state
            .bucket_metadata
            .insert(bucket.to_owned(), BucketMetadataBundle::default());
        state.buckets.insert(bucket.to_owned(), bucket_obj.clone());
        Ok(bucket_obj)
    }

    async fn head_bucket(&self, bucket: &str) -> Result<S3Bucket, Self::Error> {
        let state = self.state.read().await;
        state
            .buckets
            .get(bucket)
            .cloned()
            .ok_or_else(|| MemoryS3EngineError::BucketNotFound(bucket.to_owned()))
    }

    async fn get_bucket(&self, bucket: &str) -> Result<S3Bucket, Self::Error> {
        self.head_bucket(bucket).await
    }

    async fn list_buckets(&self) -> Result<Vec<S3Bucket>, Self::Error> {
        let state = self.state.read().await;
        let mut out: Vec<S3Bucket> = state.buckets.values().cloned().collect();
        out.sort_by(|a, b| a.identity.name.cmp(&b.identity.name));
        Ok(out)
    }

    async fn delete_bucket(&self, bucket: &str, force: bool) -> Result<(), Self::Error> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;

        let has_objects = state.objects.keys().any(|(b, _)| b == bucket);
        if has_objects && !force {
            return Err(MemoryS3EngineError::BucketNotEmpty(bucket.to_owned()));
        }

        state.objects.retain(|(b, _), _| b != bucket);
        state.multiparts.retain(|_, u| u.upload.bucket != bucket);
        state.bucket_metadata.remove(bucket);
        state.buckets.remove(bucket);
        Ok(())
    }

    async fn list_objects_v1(
        &self,
        bucket: &str,
        options: ListOptions,
    ) -> Result<ObjectListPage, Self::Error> {
        let state = self.state.read().await;
        list_objects_common(&state, bucket, &options, false)
    }

    async fn list_objects_v2(
        &self,
        bucket: &str,
        options: ListOptions,
    ) -> Result<ObjectListPage, Self::Error> {
        let state = self.state.read().await;
        list_objects_common(&state, bucket, &options, false)
    }

    async fn list_object_versions(
        &self,
        bucket: &str,
        options: ListOptions,
    ) -> Result<ObjectListPage, Self::Error> {
        let state = self.state.read().await;
        list_objects_common(&state, bucket, &options, true)
    }
}

fn list_objects_common(
    state: &crate::components::s3_engine::memory::MemoryState,
    bucket: &str,
    options: &ListOptions,
    include_versions: bool,
) -> Result<ObjectListPage, MemoryS3EngineError> {
    state.ensure_bucket(bucket)?;

    let prefix = options.prefix.as_deref().unwrap_or("");
    let delimiter = options.delimiter.as_deref();
    let max_keys = options.max_keys.map(|v| v as usize).unwrap_or(usize::MAX);

    let mut objects = Vec::new();
    let mut common_prefixes = BTreeSet::new();

    let mut keys: Vec<String> = state
        .objects
        .keys()
        .filter(|(b, _)| b == bucket)
        .map(|(_, k)| k.clone())
        .collect();
    keys.sort();

    for key in keys {
        if !key.starts_with(prefix) {
            continue;
        }

        if let Some(d) = delimiter {
            let rest = &key[prefix.len()..];
            if let Some(pos) = rest.find(d) {
                common_prefixes.insert(format!("{prefix}{}", &rest[..pos + d.len()]));
                continue;
            }
        }

        if let Some(versions) = state.objects.get(&(bucket.to_owned(), key.clone())) {
            if include_versions {
                for v in versions {
                    if !v.object.version.delete_marker {
                        objects.push(v.object.clone());
                    }
                }
            } else if let Some(v) = versions.last() {
                if !v.object.version.delete_marker {
                    objects.push(v.object.clone());
                }
            }
        }
    }

    let is_truncated = objects.len() > max_keys;
    if is_truncated {
        objects.truncate(max_keys);
    }

    Ok(ObjectListPage {
        objects,
        common_prefixes: common_prefixes.into_iter().collect(),
        next_continuation_token: None,
        next_key_marker: None,
        next_version_id_marker: None,
        is_truncated,
    })
}
