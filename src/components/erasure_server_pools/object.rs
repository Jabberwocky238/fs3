use async_trait::async_trait;
use crate::types::traits::object_layer::ObjectObjectLayer;
use crate::types::s3::object_layer_types::*;
use crate::types::s3::storage_types::*;
use crate::types::s3::core::BoxByteStream;
use crate::types::errors::S3Error;
use super::ErasureServerPools;

#[async_trait]
impl ObjectObjectLayer for ErasureServerPools {
    async fn get_object_info(&self, ctx: &Context, bucket: &str, object: &str, opts: ObjectOptions) -> Result<ObjectInfo, S3Error> {
        let version_id = opts.version_id.as_deref().unwrap_or("null");
        let fi = self.storage.read_version(ctx, bucket, object, version_id).await?;

        eprintln!("DEBUG get_object_info: size={}", fi.size);

        Ok(ObjectInfo {
            bucket: bucket.to_string(),
            name: object.to_string(),
            size: fi.size,
            etag: "".to_string(),
            content_type: "application/octet-stream".to_string(),
            user_defined: opts.user_defined,
        })
    }

    async fn get_object(&self, ctx: &Context, bucket: &str, object: &str, opts: ObjectOptions) -> Result<(ObjectInfo, BoxByteStream), S3Error> {
        let version_id = opts.version_id.as_deref().unwrap_or("null");
        let fi = self.storage.read_version(ctx, bucket, object, version_id).await?;

        let file_path = format!("{}/{}", object, fi.data_dir);
        let chunk_size = 64 * 1024;
        let (start_offset, total_size) = if let Some((start, end)) = opts.range {
            (start, end.min(fi.size - 1) + 1)
        } else {
            (0, fi.size)
        };

        use futures::stream::{self, StreamExt};
        let storage = self.storage.clone();
        let ctx_clone = ctx.clone();
        let bucket = bucket.to_string();
        let file_path = file_path.clone();

        let stream = stream::unfold((start_offset, storage, ctx_clone, bucket, file_path, total_size),
            move |(offset, storage, ctx, bucket, path, total)| async move {
                if offset >= total {
                    return None;
                }
                let read_size = std::cmp::min(chunk_size, (total - offset) as usize);
                let mut buf = vec![0u8; read_size];
                match storage.read_file(&ctx, &bucket, &path, offset as i64, &mut buf).await {
                    Ok(n) if n > 0 => {
                        buf.truncate(n as usize);
                        Some((Ok(bytes::Bytes::from(buf)), (offset + n as u64, storage, ctx, bucket, path, total)))
                    }
                    _ => None,
                }
            }).boxed();

        let info = ObjectInfo {
            bucket: ctx.request_id.clone(),
            name: object.to_string(),
            size: fi.size,
            etag: fi.version_id.clone(),
            content_type: "application/octet-stream".to_string(),
            user_defined: opts.user_defined,
        };

        Ok((info, stream))
    }

    async fn put_object(&self, ctx: &Context, bucket: &str, object: &str, data: PutObjReader, opts: ObjectOptions) -> Result<ObjectInfo, S3Error> {
        let version_id = uuid::Uuid::new_v4().to_string();
        let data_dir = uuid::Uuid::new_v4().to_string();

        let file_path = format!("{}/{}", object, data_dir);
        self.storage.create_file(ctx, bucket, &file_path, data.size, data.reader).await?;

        let fi = FileInfo {
            volume: bucket.to_string(),
            name: object.to_string(),
            version_id: version_id.clone(),
            size: data.size as u64,
            data_dir,
        };
        self.storage.write_metadata(ctx, bucket, object, fi).await?;

        Ok(ObjectInfo {
            bucket: bucket.to_string(),
            name: object.to_string(),
            size: data.size as u64,
            etag: version_id,
            content_type: "application/octet-stream".to_string(),
            user_defined: opts.user_defined,
        })
    }

    async fn copy_object(&self, ctx: &Context, src_bucket: &str, src_object: &str, dst_bucket: &str, dst_object: &str, _src_info: ObjectInfo, src_opts: ObjectOptions, dst_opts: ObjectOptions) -> Result<ObjectInfo, S3Error> {
        let src_version = src_opts.version_id.as_deref().unwrap_or("null");
        let src_fi = self.storage.read_version(ctx, src_bucket, src_object, src_version).await?;

        let src_path = format!("{}/{}", src_object, src_fi.data_dir);
        let dst_data_dir = uuid::Uuid::new_v4().to_string();
        let dst_path = format!("{}/{}", dst_object, dst_data_dir);

        let mut offset = 0i64;
        let chunk_size = 64 * 1024;
        while offset < src_fi.size as i64 {
            let read_size = std::cmp::min(chunk_size, (src_fi.size as i64 - offset) as usize);
            let mut buf = vec![0u8; read_size];
            let n = self.storage.read_file(ctx, src_bucket, &src_path, offset, &mut buf).await?;
            if n == 0 { break; }
            buf.truncate(n as usize);
            if offset == 0 {
                let stream = Box::pin(futures::stream::once(async move { Ok(bytes::Bytes::from(buf)) }));
                self.storage.create_file(ctx, dst_bucket, &dst_path, src_fi.size as i64, stream).await?;
            } else {
                self.storage.append_file(ctx, dst_bucket, &dst_path, &buf).await?;
            }
            offset += n;
        }

        let dst_fi = FileInfo {
            volume: dst_bucket.to_string(),
            name: dst_object.to_string(),
            version_id: uuid::Uuid::new_v4().to_string(),
            size: src_fi.size,
            data_dir: dst_data_dir,
        };
        self.storage.write_metadata(ctx, dst_bucket, dst_object, dst_fi.clone()).await?;

        Ok(ObjectInfo {
            bucket: dst_bucket.to_string(),
            name: dst_object.to_string(),
            size: dst_fi.size,
            etag: dst_fi.version_id,
            content_type: "application/octet-stream".to_string(),
            user_defined: dst_opts.user_defined,
        })
    }

    async fn delete_object(&self, ctx: &Context, bucket: &str, object: &str, opts: ObjectOptions) -> Result<ObjectInfo, S3Error> {
        let version_id = opts.version_id.as_deref().unwrap_or("null");
        let fi = self.storage.read_version(ctx, bucket, object, version_id).await?;

        self.storage.delete_version(ctx, bucket, object, fi.clone()).await?;

        Ok(ObjectInfo {
            bucket: bucket.to_string(),
            name: object.to_string(),
            size: fi.size,
            etag: fi.version_id,
            content_type: "".to_string(),
            user_defined: opts.user_defined,
        })
    }
}
