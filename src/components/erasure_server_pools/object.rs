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
            user_defined: fi.user_metadata,
        })
    }

    async fn get_object(&self, ctx: &Context, bucket: &str, object: &str, opts: ObjectOptions) -> Result<(ObjectInfo, BoxByteStream), S3Error> {
        let version_id = opts.version_id.as_deref().unwrap_or("null");
        let fi = self.storage.read_version(ctx, bucket, object, version_id).await?;

        eprintln!("DEBUG get_object: fi.size={}, opts.range={:?}", fi.size, opts.range);

        let file_path = format!("{}/{}", object, fi.data_dir);
        let chunk_size = 64 * 1024;
        let (start_offset, total_size) = if let Some((start, end)) = opts.range {
            (start, end + 1)
        } else {
            (0, fi.size)
        };

        eprintln!("DEBUG get_object: start_offset={}, total_size={}", start_offset, total_size);

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
            size: total_size - start_offset,
            etag: fi.version_id.clone(),
            content_type: "application/octet-stream".to_string(),
            user_defined: opts.user_defined,
        };

        Ok((info, stream))
    }

    async fn put_object(&self, ctx: &Context, bucket: &str, object: &str, data: PutObjReader, opts: ObjectOptions) -> Result<ObjectInfo, S3Error> {
        let version_id = uuid::Uuid::new_v4().to_string();
        let data_dir = uuid::Uuid::new_v4().to_string();
        let temp_object = uuid::Uuid::new_v4().to_string();
        let temp_volume = ".minio.sys/tmp";

        let temp_file_path = format!("{}/{}", temp_object, data_dir);
        let actual_size = self.storage.create_file(ctx, temp_volume, &temp_file_path, data.size, data.reader).await?;

        let fi = FileInfo {
            volume: bucket.to_string(),
            name: object.to_string(),
            version_id: version_id.clone(),
            size: actual_size,
            data_dir,
            user_metadata: opts.user_defined.clone(),
            erasure_index: 1,
            erasure_m: 1,
            erasure_n: 0,
        };
        self.storage.rename_data(ctx, temp_volume, &temp_object, fi, bucket, object).await?;

        Ok(ObjectInfo {
            bucket: bucket.to_string(),
            name: object.to_string(),
            size: actual_size,
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
            user_metadata: src_fi.user_metadata.clone(),
            erasure_index: src_fi.erasure_index,
            erasure_m: src_fi.erasure_m,
            erasure_n: src_fi.erasure_n,
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::components::xl_storage::XlStorage;
    use crate::types::traits::storage_api::{StorageMetadata, StorageVolume};

    use super::*;

    #[tokio::test]
    async fn put_object_overwrite_cleans_old_data_dir() {
        let mount_root = std::env::temp_dir().join(format!("fs3-put-object-{}", uuid::Uuid::new_v4()));
        let storage = Arc::new(XlStorage::new(mount_root.clone()));
        let object_layer = ErasureServerPools::new(storage.clone());
        let ctx = Context { request_id: "put-object-test".to_string() };
        let bucket = "bucket";
        let object = "object.txt";

        storage.make_vol(&ctx, bucket).await.unwrap();

        let first_reader = PutObjReader {
            reader: Box::pin(futures::stream::once(async { Ok(bytes::Bytes::from_static(b"first")) })),
            size: 5,
        };
        object_layer.put_object(&ctx, bucket, object, first_reader, ObjectOptions::default()).await.unwrap();
        let first = storage.read_version(&ctx, bucket, object, "null").await.unwrap();

        let second_reader = PutObjReader {
            reader: Box::pin(futures::stream::once(async { Ok(bytes::Bytes::from_static(b"second version")) })),
            size: 14,
        };
        object_layer.put_object(&ctx, bucket, object, second_reader, ObjectOptions::default()).await.unwrap();
        let second = storage.read_version(&ctx, bucket, object, "null").await.unwrap();

        assert_ne!(first.data_dir, second.data_dir);
        assert!(!mount_root.join(bucket).join(object).join(&first.data_dir).exists());
        assert!(mount_root.join(bucket).join(object).join(&second.data_dir).exists());

        let tmp_root = mount_root.join(".minio.sys").join("tmp");
        let leftovers: Vec<_> = std::fs::read_dir(&tmp_root)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name != ".trash")
            .collect();
        assert!(leftovers.is_empty(), "temporary upload paths were not cleaned: {leftovers:?}");

        let _ = std::fs::remove_dir_all(&mount_root);
    }
}
