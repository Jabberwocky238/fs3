use async_trait::async_trait;

use crate::types::errors::S3EngineError;
use crate::types::s3::core::CorsConfiguration;
use crate::types::traits::s3_engine::*;

use super::FS3Engine;

#[async_trait]
impl S3BucketWebsiteEngine<S3EngineError> for FS3Engine {
    async fn get_bucket_website(&self, bucket: &str) -> Result<Option<String>, S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context {
            request_id: "".to_string(),
        };
        let path = ".minio.sys/website.xml";
        let mut buf = vec![0u8; 4096];
        match self
            .storage
            .read_file(&ctx, bucket, path, 0, &mut buf)
            .await
        {
            Ok(n) if n > 0 => Ok(Some(
                String::from_utf8_lossy(&buf[..n as usize]).to_string(),
            )),
            _ => Ok(None),
        }
    }

    async fn put_bucket_website(&self, bucket: &str, website: String) -> Result<(), S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context {
            request_id: "".to_string(),
        };
        let path = ".minio.sys/website.xml";
        let data = website.into_bytes();
        let len = data.len() as i64;
        let stream = futures::stream::once(async move {
            Ok::<bytes::Bytes, std::io::Error>(bytes::Bytes::from(data))
        });
        self.storage
            .create_file(
                &ctx,
                bucket,
                path,
                len,
                Box::pin(stream),
                crate::types::s3::storage_types::CreateFileOptions {
                    path_kind: crate::types::s3::storage_types::StoragePathKind::Config,
                    write_kind: crate::types::s3::storage_types::StorageWriteKind::Config,
                    fsync: false,
                },
            )
            .await
            .map(|_| ())
    }

    async fn delete_bucket_website(&self, _bucket: &str) -> Result<(), S3EngineError> {
        Ok(())
    }

    async fn set_bucket_cors(
        &self,
        bucket: &str,
        cors: Option<CorsConfiguration>,
    ) -> Result<(), S3EngineError> {
        let ctx = crate::types::s3::object_layer_types::Context {
            request_id: "".to_string(),
        };
        match cors {
            Some(c) => {
                let json = serde_json::to_string(&c)?;
                self.storage.write_bucket_cors(&ctx, bucket, &json).await
            }
            None => self.storage.delete_bucket_cors(&ctx, bucket).await,
        }
    }
}
