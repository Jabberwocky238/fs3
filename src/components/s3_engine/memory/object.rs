use async_trait::async_trait;

use crate::components::s3_engine::memory::{MemoryS3Engine, MemoryS3EngineError};
use crate::types::s3::core::*;
use crate::types::traits::s3_engine::S3ObjectEngine;

#[async_trait]
impl S3ObjectEngine<MemoryS3EngineError> for MemoryS3Engine {

    async fn head_object(
        &self,
        bucket: &str,
        key: &str,
        options: ObjectReadOptions,
    ) -> Result<S3Object, MemoryS3EngineError> {
        let state = self.state.read().await;
        let versions = state.get_versions(bucket, key)?;
        let idx = crate::components::s3_engine::memory::MemoryState::select_version_index(
            versions,
            options.version_id.as_deref(),
        )
        .ok_or_else(|| MemoryS3EngineError::ObjectVersionNotFound {
            bucket: bucket.to_owned(),
            key: key.to_owned(),
            version_id: options.version_id.clone().unwrap_or_default(),
        })?;
        Ok(versions[idx].object.clone())
    }

    async fn get_object(
        &self,
        bucket: &str,
        key: &str,
        options: ObjectReadOptions,
    ) -> Result<(S3Object, Vec<u8>), MemoryS3EngineError> {
        let state = self.state.read().await;
        let versions = state.get_versions(bucket, key)?;
        let idx = crate::components::s3_engine::memory::MemoryState::select_version_index(
            versions,
            options.version_id.as_deref(),
        )
        .ok_or_else(|| MemoryS3EngineError::ObjectVersionNotFound {
            bucket: bucket.to_owned(),
            key: key.to_owned(),
            version_id: options.version_id.clone().unwrap_or_default(),
        })?;

        let obj = versions[idx].object.clone();
        let mut body = versions[idx].body.clone();
        if let Some(r) = options.range.as_deref() {
            body = MemoryS3Engine::apply_range(&body, r)?;
        }
        Ok((obj, body))
    }

    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        options: ObjectWriteOptions,
    ) -> Result<S3Object, MemoryS3EngineError> {
        let mut state = self.state.write().await;
        let etag = MemoryS3Engine::compute_etag(&body);
        state.put_object(
            bucket,
            key,
            body,
            options,
            etag,
            chrono::Utc::now(),
            MemoryS3Engine::new_version_ref(),
        )
    }

    async fn copy_object(
        &self,
        src_bucket: &str,
        src_key: &str,
        dst_bucket: &str,
        dst_key: &str,
        options: ObjectWriteOptions,
    ) -> Result<S3Object, MemoryS3EngineError> {
        let mut state = self.state.write().await;
        let src = state
            .get_versions(src_bucket, src_key)?
            .last()
            .ok_or_else(|| MemoryS3EngineError::ObjectNotFound {
                bucket: src_bucket.to_owned(),
                key: src_key.to_owned(),
            })?
            .clone();
        let etag = MemoryS3Engine::compute_etag(&src.body);
        state.put_object(
            dst_bucket,
            dst_key,
            src.body.clone(),
            options,
            etag,
            chrono::Utc::now(),
            MemoryS3Engine::new_version_ref(),
        )
    }

    async fn delete_object(
        &self,
        bucket: &str,
        key: &str,
        options: DeleteObjectOptions,
    ) -> Result<ObjectVersionRef, MemoryS3EngineError> {
        let mut state = self.state.write().await;
        let versions = state.get_versions_mut(bucket, key)?;
        let idx = crate::components::s3_engine::memory::MemoryState::select_version_index(
            versions,
            options.version_id.as_deref(),
        )
        .ok_or_else(|| MemoryS3EngineError::ObjectVersionNotFound {
            bucket: bucket.to_owned(),
            key: key.to_owned(),
            version_id: options.version_id.clone().unwrap_or_default(),
        })?;

        let removed = versions.remove(idx);
        if let Some(last) = versions.last_mut() {
            last.object.version.is_latest = true;
        }
        if versions.is_empty() {
            state.objects.remove(&(bucket.to_owned(), key.to_owned()));
        }
        Ok(removed.object.version)
    }

    async fn delete_objects(
        &self,
        bucket: &str,
        keys: Vec<String>,
        options: DeleteObjectOptions,
    ) -> Result<DeleteResult, MemoryS3EngineError> {
        let mut deleted = Vec::new();
        let mut errors = Vec::new();

        for key in keys {
            match self.delete_object(bucket, &key, options.clone()).await {
                Ok(v) => deleted.push(v),
                Err(e) => errors.push(S3ErrorDetail {
                    code: "DeleteFailed".to_owned(),
                    message: e.to_string(),
                    key: Some(key),
                    version_id: options.version_id.clone(),
                }),
            }
        }
        Ok(DeleteResult { deleted, errors })
    }

    async fn get_object_tagging(&self, bucket: &str, key: &str) -> Result<TagMap, MemoryS3EngineError> {
        let state = self.state.read().await;
        let versions = state.get_versions(bucket, key)?;
        Ok(versions
            .last()
            .ok_or_else(|| MemoryS3EngineError::ObjectNotFound {
                bucket: bucket.to_owned(),
                key: key.to_owned(),
            })?
            .object
            .user_tags
            .clone())
    }

    async fn put_object_tagging(
        &self,
        bucket: &str,
        key: &str,
        tags: TagMap,
    ) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        let latest = state
            .get_versions_mut(bucket, key)?
            .last_mut()
            .ok_or_else(|| MemoryS3EngineError::ObjectNotFound {
                bucket: bucket.to_owned(),
                key: key.to_owned(),
            })?;
        latest.object.user_tags = tags;
        Ok(())
    }

    async fn delete_object_tagging(&self, bucket: &str, key: &str) -> Result<(), MemoryS3EngineError> {
        self.put_object_tagging(bucket, key, TagMap::new()).await
    }

    async fn get_object_retention(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<Option<ObjectRetention>, MemoryS3EngineError> {
        let state = self.state.read().await;
        let latest = state
            .get_versions(bucket, key)?
            .last()
            .ok_or_else(|| MemoryS3EngineError::ObjectNotFound {
                bucket: bucket.to_owned(),
                key: key.to_owned(),
            })?;
        Ok(latest.object.retention.clone())
    }

    async fn put_object_retention(
        &self,
        bucket: &str,
        key: &str,
        retention: ObjectRetention,
    ) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        let latest = state
            .get_versions_mut(bucket, key)?
            .last_mut()
            .ok_or_else(|| MemoryS3EngineError::ObjectNotFound {
                bucket: bucket.to_owned(),
                key: key.to_owned(),
            })?;
        latest.object.retention = Some(retention);
        Ok(())
    }

    async fn get_object_legal_hold(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<Option<ObjectLegalHold>, MemoryS3EngineError> {
        let state = self.state.read().await;
        let latest = state
            .get_versions(bucket, key)?
            .last()
            .ok_or_else(|| MemoryS3EngineError::ObjectNotFound {
                bucket: bucket.to_owned(),
                key: key.to_owned(),
            })?;
        Ok(latest.object.legal_hold.clone())
    }

    async fn put_object_legal_hold(
        &self,
        bucket: &str,
        key: &str,
        legal_hold: ObjectLegalHold,
    ) -> Result<(), MemoryS3EngineError> {
        let mut state = self.state.write().await;
        let latest = state
            .get_versions_mut(bucket, key)?
            .last_mut()
            .ok_or_else(|| MemoryS3EngineError::ObjectNotFound {
                bucket: bucket.to_owned(),
                key: key.to_owned(),
            })?;
        latest.object.legal_hold = Some(legal_hold);
        Ok(())
    }
}
