use async_trait::async_trait;

use crate::components::s3_engine::memory::state::{MultipartPartData, MultipartUploadState};
use crate::components::s3_engine::memory::{MemoryS3Engine, MemoryS3EngineError};
use crate::types::s3::core::*;
use crate::types::traits::s3_engine::S3MultipartEngine;

#[async_trait]
impl S3MultipartEngine for MemoryS3Engine {
    type Error = MemoryS3EngineError;

    async fn new_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        options: ObjectWriteOptions,
    ) -> Result<MultipartUpload, Self::Error> {
        let mut state = self.state.write().await;
        state.ensure_bucket(bucket)?;

        let upload = MultipartUpload {
            bucket: bucket.to_owned(),
            key: key.to_owned(),
            upload_id: uuid::Uuid::new_v4().to_string(),
            initiated_at: chrono::Utc::now(),
            storage_class: options.storage_class.clone(),
            user_metadata: options.user_metadata.clone(),
            user_tags: options.user_tags.clone(),
        };

        state.multiparts.insert(
            upload.upload_id.clone(),
            MultipartUploadState {
                upload: upload.clone(),
                parts: std::collections::BTreeMap::new(),
                write_options: options,
            },
        );

        Ok(upload)
    }

    async fn put_object_part(
        &self,
        _bucket: &str,
        _key: &str,
        upload_id: &str,
        part_number: u32,
        body: Vec<u8>,
    ) -> Result<UploadedPart, Self::Error> {
        let mut state = self.state.write().await;
        let upload = state
            .multiparts
            .get_mut(upload_id)
            .ok_or_else(|| MemoryS3EngineError::MultipartNotFound(upload_id.to_owned()))?;

        let uploaded = UploadedPart {
            part_number,
            etag: MemoryS3Engine::compute_etag(&body),
            size: body.len() as u64,
        };

        upload.parts.insert(
            part_number,
            MultipartPartData {
                uploaded: uploaded.clone(),
                body,
            },
        );

        Ok(uploaded)
    }

    async fn copy_object_part(
        &self,
        src_bucket: &str,
        src_key: &str,
        _dst_bucket: &str,
        _dst_key: &str,
        upload_id: &str,
        part_number: u32,
    ) -> Result<UploadedPart, Self::Error> {
        let mut state = self.state.write().await;
        let src_body = state
            .get_versions(src_bucket, src_key)?
            .last()
            .ok_or_else(|| MemoryS3EngineError::ObjectNotFound {
                bucket: src_bucket.to_owned(),
                key: src_key.to_owned(),
            })?
            .body
            .clone();

        let upload = state
            .multiparts
            .get_mut(upload_id)
            .ok_or_else(|| MemoryS3EngineError::MultipartNotFound(upload_id.to_owned()))?;

        let uploaded = UploadedPart {
            part_number,
            etag: MemoryS3Engine::compute_etag(&src_body),
            size: src_body.len() as u64,
        };

        upload.parts.insert(
            part_number,
            MultipartPartData {
                uploaded: uploaded.clone(),
                body: src_body,
            },
        );

        Ok(uploaded)
    }

    async fn list_object_parts(
        &self,
        _bucket: &str,
        _key: &str,
        upload_id: &str,
    ) -> Result<Vec<UploadedPart>, Self::Error> {
        let state = self.state.read().await;
        let upload = state
            .multiparts
            .get(upload_id)
            .ok_or_else(|| MemoryS3EngineError::MultipartNotFound(upload_id.to_owned()))?;
        Ok(upload.parts.values().map(|v| v.uploaded.clone()).collect())
    }

    async fn complete_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        completed: CompleteMultipartInput,
    ) -> Result<S3Object, Self::Error> {
        let mut state = self.state.write().await;
        let upload = state
            .multiparts
            .remove(upload_id)
            .ok_or_else(|| MemoryS3EngineError::MultipartNotFound(upload_id.to_owned()))?;

        let ordered = if completed.parts.is_empty() {
            upload
                .parts
                .values()
                .map(|p| p.uploaded.clone())
                .collect::<Vec<_>>()
        } else {
            completed.parts
        };

        let mut body = Vec::new();
        for p in &ordered {
            let part = upload.parts.get(&p.part_number).ok_or_else(|| {
                MemoryS3EngineError::MultipartPartMissing {
                    upload_id: upload_id.to_owned(),
                    part_number: p.part_number,
                }
            })?;
            body.extend_from_slice(&part.body);
        }
        let etag = MemoryS3Engine::compute_etag(&body);

        let obj = state.put_object(
            bucket,
            key,
            body,
            upload.write_options,
            etag,
            chrono::Utc::now(),
            MemoryS3Engine::new_version_ref(),
        )?;

        let mut final_obj = obj;
        final_obj.parts = ordered
            .into_iter()
            .map(|p| ObjectPart {
                part_number: p.part_number,
                etag: p.etag,
                size: p.size,
                checksum: None,
                last_modified: Some(chrono::Utc::now()),
            })
            .collect();

        if let Some(last) = state
            .objects
            .get_mut(&(bucket.to_owned(), key.to_owned()))
            .and_then(|v| v.last_mut())
        {
            last.object = final_obj.clone();
        }

        Ok(final_obj)
    }

    async fn abort_multipart_upload(
        &self,
        _bucket: &str,
        _key: &str,
        upload_id: &str,
    ) -> Result<(), Self::Error> {
        let mut state = self.state.write().await;
        if state.multiparts.remove(upload_id).is_some() {
            Ok(())
        } else {
            Err(MemoryS3EngineError::MultipartNotFound(upload_id.to_owned()))
        }
    }

    async fn list_multipart_uploads(
        &self,
        bucket: &str,
        options: ListOptions,
    ) -> Result<Vec<MultipartUpload>, Self::Error> {
        let state = self.state.read().await;
        state.ensure_bucket(bucket)?;
        let prefix = options.prefix.unwrap_or_default();

        let mut out: Vec<MultipartUpload> = state
            .multiparts
            .values()
            .filter(|u| u.upload.bucket == bucket && u.upload.key.starts_with(&prefix))
            .map(|u| u.upload.clone())
            .collect();
        out.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(out)
    }
}
