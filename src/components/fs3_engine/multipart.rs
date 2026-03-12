use async_trait::async_trait;

use crate::types::errors::S3EngineError;
use crate::types::s3::core::*;
use crate::types::traits::s3_engine::*;

use super::FS3Engine;

#[async_trait]
impl S3MultipartEngine<S3EngineError> for FS3Engine {
    async fn new_multipart_upload(&self, bucket: &str, key: &str, _options: ObjectWriteOptions) -> Result<MultipartUpload, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let result = self.object_layer.new_multipart_upload(&ctx, bucket, key, Default::default()).await?;
        Ok(MultipartUpload {
            bucket: bucket.to_string(),
            key: key.to_string(),
            upload_id: result.upload_id,
            initiated_at: chrono::Utc::now(),
            storage_class: Default::default(),
            user_metadata: Default::default(),
            user_tags: Default::default(),
        })
    }

    async fn put_object_part(&self, bucket: &str, key: &str, upload_id: &str, part_number: u32, body: BoxByteStream) -> Result<UploadedPart, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let size = body.size_hint().0 as i64;
        let data = crate::types::s3::storage_types::PutObjReader { reader: body, size };
        let result = self.object_layer.put_object_part(&ctx, bucket, key, upload_id, part_number, data, Default::default()).await?;
        Ok(UploadedPart { part_number, etag: result.etag, size: result.size })
    }

    async fn copy_object_part(&self, _src_bucket: &str, _src_key: &str, _dst_bucket: &str, _dst_key: &str, _upload_id: &str, _part_number: u32) -> Result<UploadedPart, S3EngineError> {
        Err(S3EngineError::from("not implemented"))
    }

    async fn list_object_parts(&self, _bucket: &str, _key: &str, _upload_id: &str) -> Result<Vec<UploadedPart>, S3EngineError> {
        Ok(Vec::new())
    }

    async fn complete_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str, completed: CompleteMultipartInput) -> Result<S3Object, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let parts = completed.parts.into_iter().map(|p| crate::types::s3::storage_types::CompletePart {
            part_number: p.part_number,
            etag: p.etag,
        }).collect();
        let result = self.object_layer.complete_multipart_upload(&ctx, bucket, key, upload_id, parts, Default::default()).await?;
        Ok(S3Object {
            bucket: bucket.to_string(),
            key: key.to_string(),
            size: result.size,
            etag: result.etag,
            last_modified: chrono::Utc::now(),
            content_type: Some(result.content_type),
            content_encoding: None,
            storage_class: Default::default(),
            user_metadata: Default::default(),
            user_tags: Default::default(),
            version: Default::default(),
            parts: Default::default(),
            checksums: Default::default(),
            replication_state: Default::default(),
            retention: None,
            legal_hold: None,
            restore_expiry: None,
            restore_ongoing: false,
        })
    }

    async fn abort_multipart_upload(&self, _bucket: &str, _key: &str, _upload_id: &str) -> Result<(), S3EngineError> {
        Ok(())
    }

    async fn list_multipart_uploads(&self, _bucket: &str, _options: ListOptions) -> Result<Vec<MultipartUpload>, S3EngineError> {
        Ok(Vec::new())
    }
}
