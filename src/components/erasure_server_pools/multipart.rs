use async_trait::async_trait;
use crate::types::traits::object_layer::ObjectMultipartLayer;
use crate::types::s3::object_layer_types::*;
use crate::types::s3::storage_types::*;
use crate::types::errors::S3Error;
use super::ErasureServerPools;

#[async_trait]
impl ObjectMultipartLayer for ErasureServerPools {
    async fn new_multipart_upload(&self, _ctx: &Context, _bucket: &str, _object: &str, _opts: ObjectOptions) -> Result<NewMultipartUploadResult, S3Error> {
        Ok(NewMultipartUploadResult {
            upload_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    async fn put_object_part(&self, ctx: &Context, bucket: &str, object: &str, upload_id: &str, part_id: u32, data: PutObjReader, _opts: ObjectOptions) -> Result<PartInfo, S3Error> {
        let part_path = format!("{}/.minio.sys/multipart/{}/part.{}", object, upload_id, part_id);
        self.storage.create_file(ctx, bucket, &part_path, data.size, data.reader).await?;

        Ok(PartInfo {
            part_number: part_id,
            etag: uuid::Uuid::new_v4().to_string(),
            size: data.size as u64,
        })
    }

    async fn complete_multipart_upload(&self, _ctx: &Context, _bucket: &str, _object: &str, _upload_id: &str, _parts: Vec<CompletePart>, _opts: ObjectOptions) -> Result<ObjectInfo, S3Error> {
        todo!()
    }

    async fn abort_multipart_upload(&self, _ctx: &Context, _bucket: &str, _object: &str, _upload_id: &str, _opts: ObjectOptions) -> Result<(), S3Error> {
        Ok(())
    }
}
