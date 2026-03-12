use super::ErasureServerPools;
use crate::types::errors::FS3Error;
use crate::types::s3::object_layer_types::*;
use crate::types::s3::storage_types::*;
use crate::types::traits::object_layer::{ObjectMultipartLayer, ObjectObjectLayer};
use async_trait::async_trait;

#[async_trait]
impl ObjectMultipartLayer<FS3Error> for ErasureServerPools {
    async fn new_multipart_upload(
        &self,
        _ctx: &Context,
        _bucket: &str,
        _object: &str,
        _opts: ObjectOptions,
    ) -> Result<NewMultipartUploadResult, FS3Error> {
        Ok(NewMultipartUploadResult {
            upload_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    async fn put_object_part(
        &self,
        ctx: &Context,
        bucket: &str,
        _object: &str,
        upload_id: &str,
        part_id: u32,
        data: PutObjReader,
        _opts: ObjectOptions,
    ) -> Result<PartInfo, FS3Error> {
        let part_path = format!("tmp/multipart/{}/part.{}", upload_id, part_id);
        self.storage
            .create_file(
                ctx,
                bucket,
                &part_path,
                data.size,
                data.reader,
                CreateFileOptions {
                    path_kind: StoragePathKind::Temporary,
                    write_kind: StorageWriteKind::Data,
                    fsync: false,
                },
            )
            .await?;

        Ok(PartInfo {
            part_number: part_id,
            etag: uuid::Uuid::new_v4().to_string(),
            size: data.size as u64,
        })
    }

    async fn complete_multipart_upload(
        &self,
        ctx: &Context,
        bucket: &str,
        object: &str,
        upload_id: &str,
        parts: Vec<CompletePart>,
        _opts: ObjectOptions,
    ) -> Result<ObjectInfo, FS3Error> {
        use futures::stream::{self};

        let mut all_data = Vec::new();

        for part in parts.iter() {
            let part_path = format!("tmp/multipart/{}/part.{}", upload_id, part.part_number);
            let mut buf = vec![0u8; 8192];
            let mut offset = 0i64;
            loop {
                let n = self
                    .storage
                    .read_file(ctx, bucket, &part_path, offset, &mut buf)
                    .await?;
                if n == 0 {
                    break;
                }
                all_data.extend_from_slice(&buf[..n as usize]);
                offset += n;
            }
        }

        let size = all_data.len() as i64;
        let stream = stream::once(async move {
            Ok::<bytes::Bytes, std::io::Error>(bytes::Bytes::from(all_data))
        });
        let reader = PutObjReader {
            reader: Box::pin(stream),
            size,
        };

        self.put_object(ctx, bucket, object, reader, Default::default())
            .await
    }

    async fn abort_multipart_upload(
        &self,
        _ctx: &Context,
        _bucket: &str,
        _object: &str,
        _upload_id: &str,
        _opts: ObjectOptions,
    ) -> Result<(), FS3Error> {
        Ok(())
    }
}
