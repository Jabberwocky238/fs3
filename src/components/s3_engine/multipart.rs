use async_trait::async_trait;
use futures::TryStreamExt;

use crate::types::s3::core::*;
use crate::types::traits::s3_engine::S3MultipartEngine;
use crate::types::traits::s3_metadata_storage::*;
use crate::types::traits::s3_mount::*;

use super::{S3EngineImpl, S3EngineImplError};

#[async_trait]
impl<S, M> S3MultipartEngine<S3EngineImplError> for S3EngineImpl<S, M>
where
    S: S3MetadataStorageBucket<S3EngineImplError>
        + S3MetadataStorageObject<S3EngineImplError>
        + S3MetadataStorageMultipart<S3EngineImplError>
        + Send + Sync,
    M: S3MountRead<S3EngineImplError>
        + S3MountWrite<S3EngineImplError>
        + S3MountMultipart<S3EngineImplError>
        + Send + Sync,
{
    async fn new_multipart_upload(&self, bucket: &str, key: &str, options: ObjectWriteOptions) -> Result<MultipartUpload, S3EngineImplError> {
        self.metadata.load_bucket(bucket).await?
            .ok_or_else(|| S3EngineImplError::BucketNotFound(bucket.to_owned()))?;
        let upload = MultipartUpload {
            bucket: bucket.to_owned(),
            key: key.to_owned(),
            upload_id: uuid::Uuid::new_v4().to_string(),
            initiated_at: chrono::Utc::now(),
            storage_class: options.storage_class.clone(),
            user_metadata: options.user_metadata.clone(),
            user_tags: options.user_tags.clone(),
        };
        self.metadata.store_multipart(&upload).await?;
        Ok(upload)
    }
