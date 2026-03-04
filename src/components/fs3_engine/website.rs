use async_trait::async_trait;
use crate::types::traits::s3_engine::*;
use crate::types::errors::S3EngineError;
use super::FS3Engine;

#[async_trait]
impl S3BucketWebsiteEngine for FS3Engine {
    async fn get_bucket_website(&self, bucket: &str) -> Result<Option<String>, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let path = format!("{}/.minio.sys/website.xml", bucket);
        let mut buf = vec![0u8; 4096];
        match self.storage.read_file(&ctx, bucket, &path, 0, &mut buf).await {
            Ok(n) if n > 0 => Ok(Some(String::from_utf8_lossy(&buf[..n as usize]).to_string())),
            _ => Ok(None)
        }
    }

    async fn put_bucket_website(&self, bucket: &str, website: String) -> Result<(), S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context { request_id: "".to_string() };
        let path = format!("{}/.minio.sys/website.xml", bucket);
        let data = website.into_bytes();
        let len = data.len() as i64;
        let stream = futures::stream::once(async move { Ok::<bytes::Bytes, std::io::Error>(bytes::Bytes::from(data)) });
        self.storage.create_file(&ctx, bucket, &path, len, Box::pin(stream)).await
            .map_err(|e| S3EngineError::Storage(e.to_string()))
    }

    async fn delete_bucket_website(&self, bucket: &str) -> Result<(), S3EngineError> {
        Ok(())
    }
}
