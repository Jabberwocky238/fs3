use async_trait::async_trait;
use crate::types::traits::s3_engine::*;
use crate::types::s3::core::*;
use crate::types::errors::S3EngineError;
use super::FS3Engine;

#[async_trait]
impl S3MultipartEngine for FS3Engine {
    async fn new_multipart_upload(&self, bucket: &str, key: &str, _options: ObjectWriteOptions) -> Result<MultipartUpload, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let result = self.object_layer.new_multipart_upload(&ctx, bucket, key, Default::default()).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))?;
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

    async fn put_object_part(&self, _bucket: &str, _key: &str, _upload_id: &str, _part_number: u32, _body: BoxByteStream) -> Result<UploadedPart, S3EngineError> {
        Err(S3EngineError::Storage("not implemented".to_string()))
    }

    async fn copy_object_part(&self, _src_bucket: &str, _src_key: &str, _dst_bucket: &str, _dst_key: &str, _upload_id: &str, _part_number: u32) -> Result<UploadedPart, S3EngineError> {
        Err(S3EngineError::Storage("not implemented".to_string()))
    }

    async fn list_object_parts(&self, _bucket: &str, _key: &str, _upload_id: &str) -> Result<Vec<UploadedPart>, S3EngineError> {
        Ok(Vec::new())
    }

    async fn complete_multipart_upload(&self, _bucket: &str, _key: &str, _upload_id: &str, _completed: CompleteMultipartInput) -> Result<S3Object, S3EngineError> {
        Err(S3EngineError::Storage("not implemented".to_string()))
    }

    async fn abort_multipart_upload(&self, _bucket: &str, _key: &str, _upload_id: &str) -> Result<(), S3EngineError> {
        Ok(())
    }

    async fn list_multipart_uploads(&self, _bucket: &str, _options: ListOptions) -> Result<Vec<MultipartUpload>, S3EngineError> {
        Ok(Vec::new())
    }
}
