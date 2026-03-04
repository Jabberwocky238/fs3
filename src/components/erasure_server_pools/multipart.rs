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
        let part_path = format!("_multipart/{}/part.{}", upload_id, part_id);
        self.storage.create_file(ctx, bucket, &part_path, data.size, data.reader).await?;

        Ok(PartInfo {
            part_number: part_id,
            etag: uuid::Uuid::new_v4().to_string(),
            size: data.size as u64,
        })
    }

    async fn complete_multipart_upload(&self, ctx: &Context, bucket: &str, object: &str, upload_id: &str, parts: Vec<CompletePart>, opts: ObjectOptions) -> Result<ObjectInfo, S3Error> {
        use futures::stream::{self, StreamExt};
        use bytes::Bytes;

        let mut total_size = 0u64;
        let mut all_data = Vec::new();

        for part in parts.iter() {
            let part_path = format!("_multipart/{}/part.{}", upload_id, part.part_number);
            let mut buf = vec![0u8; 8192];
            let mut offset = 0i64;
            loop {
                let n = self.storage.read_file(ctx, bucket, &part_path, offset, &mut buf).await?;
                if n == 0 { break; }
                all_data.extend_from_slice(&buf[..n as usize]);
                offset += n;
                total_size += n as u64;
            }
        }

        let size = all_data.len() as i64;
        let stream = stream::once(async move { Ok::<Bytes, std::io::Error>(Bytes::from(all_data)) });
        let reader = Box::pin(stream);

        self.storage.create_file(ctx, bucket, object, size, reader).await?;

        Ok(ObjectInfo {
            bucket: bucket.to_string(),
            name: object.to_string(),
            etag: uuid::Uuid::new_v4().to_string(),
            size: total_size,
            content_type: String::new(),
            user_defined: Default::default(),
        })
    }

    async fn abort_multipart_upload(&self, _ctx: &Context, _bucket: &str, _object: &str, _upload_id: &str, _opts: ObjectOptions) -> Result<(), S3Error> {
        Ok(())
    }
}
