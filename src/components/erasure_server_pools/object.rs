use super::ErasureServerPools;
use super::write_path::{
    collect_stream, decode_bitrot_frames, encode_bitrot_frames, part_relative_path,
    to_single_chunk_stream,
};
use crate::types::errors::FS3Error;
use crate::types::s3::core::BoxByteStream;
use crate::types::s3::object_layer_types::*;
use crate::types::s3::storage_types::*;
use crate::types::traits::object_layer::ObjectObjectLayer;
use async_trait::async_trait;

#[async_trait]
impl ObjectObjectLayer<FS3Error> for ErasureServerPools {
    async fn get_object_info(
        &self,
        ctx: &Context,
        bucket: &str,
        object: &str,
        opts: ObjectOptions,
    ) -> Result<ObjectInfo, FS3Error> {
        let version_id = opts.version_id.as_deref().unwrap_or("null");
        let fi = self
            .storage
            .read_version(ctx, bucket, object, version_id)
            .await?;

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

    async fn get_object(
        &self,
        ctx: &Context,
        bucket: &str,
        object: &str,
        opts: ObjectOptions,
    ) -> Result<(ObjectInfo, BoxByteStream), FS3Error> {
        let version_id = opts.version_id.as_deref().unwrap_or("null");
        let fi = self
            .storage
            .read_version(ctx, bucket, object, version_id)
            .await?;

        eprintln!(
            "DEBUG get_object: fi.size={}, opts.range={:?}",
            fi.size, opts.range
        );

        let file_path = format!("{}/{}", object, part_relative_path(&fi.data_dir, 1));
        let encoded = read_all(&*self.storage, ctx, bucket, &file_path).await?;
        let raw = decode_bitrot_frames(&encoded, fi.size)?;
        let (start_offset, total_size) = if let Some((start, end)) = opts.range {
            (start as usize, (end + 1) as usize)
        } else {
            (0usize, fi.size as usize)
        };
        let body = raw
            .get(start_offset..std::cmp::min(total_size, raw.len()))
            .unwrap_or(&[])
            .to_vec();
        let stream = to_single_chunk_stream(body);

        let info = ObjectInfo {
            bucket: ctx.request_id.clone(),
            name: object.to_string(),
            size: (total_size.saturating_sub(start_offset)) as u64,
            etag: fi.version_id.clone(),
            content_type: "application/octet-stream".to_string(),
            user_defined: opts.user_defined,
        };

        Ok((info, stream))
    }

    async fn put_object(
        &self,
        ctx: &Context,
        bucket: &str,
        object: &str,
        data: PutObjReader,
        opts: ObjectOptions,
    ) -> Result<ObjectInfo, FS3Error> {
        let version_id = uuid::Uuid::new_v4().to_string();
        let data_dir = uuid::Uuid::new_v4().to_string();
        let temp_object = uuid::Uuid::new_v4().to_string();
        let temp_volume = ".minio.sys/tmp";

        let raw = collect_stream(data.reader).await?;
        let actual_size = raw.len() as u64;
        let encoded = encode_bitrot_frames(&raw);
        let data_path = format!("{}/{}", temp_object, part_relative_path(&data_dir, 1));
        self.storage
            .create_file(
                ctx,
                temp_volume,
                &data_path,
                encoded.len() as i64,
                to_single_chunk_stream(encoded),
                CreateFileOptions {
                    path_kind: StoragePathKind::Temporary,
                    write_kind: StorageWriteKind::Data,
                    fsync: false,
                },
            )
            .await?;

        let fi = FileInfo {
            volume: bucket.to_string(),
            name: object.to_string(),
            version_id: version_id.clone(),
            size: actual_size,
            data_dir: data_dir.clone(),
            etag: String::new(),
            content_type: "application/octet-stream".to_string(),
            user_metadata: opts.user_defined.clone(),
            erasure_index: 1,
            erasure_m: 1,
            erasure_n: 0,
        };
        let commit = self
            .storage
            .rename_data(
                ctx,
                temp_volume,
                &temp_object,
                fi,
                bucket,
                object,
                RenameDataOptions {
                    path_kind: StoragePathKind::Temporary,
                    defer_old_data_dir_cleanup: true,
                    defer_src_path_cleanup: true,
                },
            )
            .await?;

        if let Some(old_path) = &commit.old_data_path {
            self.storage
                .delete_path(
                    ctx,
                    bucket,
                    old_path,
                    DeletePathOptions {
                        recursive: true,
                        ignore_not_found: true,
                    },
                )
                .await?;
        }

        if !commit.cleanup_src_path.is_empty() {
            self.storage
                .delete_path(
                    ctx,
                    &commit.cleanup_src_volume,
                    &commit.cleanup_src_path,
                    DeletePathOptions {
                        recursive: true,
                        ignore_not_found: true,
                    },
                )
                .await?;
        }

        Ok(ObjectInfo {
            bucket: bucket.to_string(),
            name: object.to_string(),
            size: actual_size,
            etag: version_id,
            content_type: "application/octet-stream".to_string(),
            user_defined: opts.user_defined,
        })
    }

    async fn copy_object(
        &self,
        ctx: &Context,
        src_bucket: &str,
        src_object: &str,
        dst_bucket: &str,
        dst_object: &str,
        _src_info: ObjectInfo,
        src_opts: ObjectOptions,
        dst_opts: ObjectOptions,
    ) -> Result<ObjectInfo, FS3Error> {
        let src_version = src_opts.version_id.as_deref().unwrap_or("null");
        let src_fi = self
            .storage
            .read_version(ctx, src_bucket, src_object, src_version)
            .await?;

        let src_path = format!("{}/{}", src_object, part_relative_path(&src_fi.data_dir, 1));
        let dst_data_dir = uuid::Uuid::new_v4().to_string();
        let dst_path = format!("{}/{}", dst_object, part_relative_path(&dst_data_dir, 1));

        let mut offset = 0i64;
        let chunk_size = 64 * 1024;
        while offset < src_fi.size as i64 {
            let read_size = std::cmp::min(chunk_size, (src_fi.size as i64 - offset) as usize);
            let mut buf = vec![0u8; read_size];
            let n = self
                .storage
                .read_file(ctx, src_bucket, &src_path, offset, &mut buf)
                .await?;
            if n == 0 {
                break;
            }
            buf.truncate(n as usize);
            if offset == 0 {
                let stream = Box::pin(futures::stream::once(
                    async move { Ok(bytes::Bytes::from(buf)) },
                ));
                self.storage
                    .create_file(
                        ctx,
                        dst_bucket,
                        &dst_path,
                        src_fi.size as i64,
                        stream,
                        CreateFileOptions::default(),
                    )
                    .await?;
            } else {
                self.storage
                    .append_file(ctx, dst_bucket, &dst_path, &buf)
                    .await?;
            }
            offset += n;
        }

        let dst_fi = FileInfo {
            volume: dst_bucket.to_string(),
            name: dst_object.to_string(),
            version_id: uuid::Uuid::new_v4().to_string(),
            size: src_fi.size,
            data_dir: dst_data_dir,
            etag: String::new(),
            content_type: "application/octet-stream".to_string(),
            user_metadata: src_fi.user_metadata.clone(),
            erasure_index: src_fi.erasure_index,
            erasure_m: 1,
            erasure_n: 0,
        };
        self.storage
            .write_metadata(ctx, dst_bucket, dst_object, dst_fi.clone())
            .await?;

        Ok(ObjectInfo {
            bucket: dst_bucket.to_string(),
            name: dst_object.to_string(),
            size: dst_fi.size,
            etag: dst_fi.version_id,
            content_type: "application/octet-stream".to_string(),
            user_defined: dst_opts.user_defined,
        })
    }

    async fn delete_object(
        &self,
        ctx: &Context,
        bucket: &str,
        object: &str,
        opts: ObjectOptions,
    ) -> Result<ObjectInfo, FS3Error> {
        let version_id = opts.version_id.as_deref().unwrap_or("null");
        let fi = self
            .storage
            .read_version(ctx, bucket, object, version_id)
            .await?;

        self.storage
            .delete_version(ctx, bucket, object, fi.clone())
            .await?;

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

async fn read_all(
    storage: &dyn crate::types::traits::storage_api::StorageAPI<FS3Error>,
    ctx: &Context,
    bucket: &str,
    path: &str,
) -> Result<Vec<u8>, FS3Error> {
    let mut offset = 0i64;
    let mut out = Vec::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = storage
            .read_file(ctx, bucket, path, offset, &mut buf)
            .await?;
        if n <= 0 {
            break;
        }
        out.extend_from_slice(&buf[..n as usize]);
        offset += n;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::components::xl_storage::XlStorage;
    use crate::types::traits::storage_api::{StorageMetadata, StorageVolume};

    use super::*;

    #[tokio::test]
    async fn put_object_overwrite_cleans_old_data_dir() {
        let mount_root =
            std::env::temp_dir().join(format!("fs3-put-object-{}", uuid::Uuid::new_v4()));
        let storage = Arc::new(XlStorage::new(mount_root.clone()));
        let object_layer = ErasureServerPools::new(storage.clone());
        let ctx = Context {
            request_id: "put-object-test".to_string(),
        };
        let bucket = "bucket";
        let object = "object.txt";

        storage.make_vol(&ctx, bucket).await.unwrap();

        let first_reader = PutObjReader {
            reader: Box::pin(futures::stream::once(async {
                Ok(bytes::Bytes::from_static(b"first"))
            })),
            size: 5,
        };
        object_layer
            .put_object(&ctx, bucket, object, first_reader, ObjectOptions::default())
            .await
            .unwrap();
        let first = storage
            .read_version(&ctx, bucket, object, "null")
            .await
            .unwrap();

        let second_reader = PutObjReader {
            reader: Box::pin(futures::stream::once(async {
                Ok(bytes::Bytes::from_static(b"second version"))
            })),
            size: 14,
        };
        object_layer
            .put_object(
                &ctx,
                bucket,
                object,
                second_reader,
                ObjectOptions::default(),
            )
            .await
            .unwrap();
        let second = storage
            .read_version(&ctx, bucket, object, "null")
            .await
            .unwrap();

        assert_ne!(first.data_dir, second.data_dir);
        assert!(
            !mount_root
                .join(bucket)
                .join(object)
                .join(&first.data_dir)
                .exists()
        );
        assert!(
            mount_root
                .join(bucket)
                .join(object)
                .join(&second.data_dir)
                .exists()
        );

        let tmp_root = mount_root.join(".minio.sys").join("tmp");
        let leftovers: Vec<_> = std::fs::read_dir(&tmp_root)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name != ".trash")
            .collect();
        assert!(
            leftovers.is_empty(),
            "temporary upload paths were not cleaned: {leftovers:?}"
        );

        let _ = std::fs::remove_dir_all(&mount_root);
    }

    #[tokio::test]
    async fn put_object_uses_minio_part_layout_and_round_trips() {
        let mount_root =
            std::env::temp_dir().join(format!("fs3-put-object-layout-{}", uuid::Uuid::new_v4()));
        let storage = Arc::new(XlStorage::new(mount_root.clone()));
        let object_layer = ErasureServerPools::new(storage.clone());
        let ctx = Context {
            request_id: "put-object-layout-test".to_string(),
        };
        let bucket = "bucket";
        let object = "object.txt";
        let body = b"layout-compatible-body".to_vec();
        let body_for_stream = body.clone();

        storage.make_vol(&ctx, bucket).await.unwrap();

        object_layer
            .put_object(
                &ctx,
                bucket,
                object,
                PutObjReader {
                    reader: Box::pin(futures::stream::once(async move {
                        Ok(bytes::Bytes::from(body_for_stream))
                    })),
                    size: body.len() as i64,
                },
                ObjectOptions::default(),
            )
            .await
            .unwrap();

        let fi = storage
            .read_version(&ctx, bucket, object, "null")
            .await
            .unwrap();
        let data_dir = mount_root.join(bucket).join(object).join(&fi.data_dir);
        assert!(data_dir.join("part.1").exists());
        assert!(!data_dir.join("shards").exists());

        let (_info, stream) = object_layer
            .get_object(&ctx, bucket, object, ObjectOptions::default())
            .await
            .unwrap();
        let got: Vec<bytes::Bytes> = futures::TryStreamExt::try_collect(stream).await.unwrap();
        assert_eq!(got.concat(), b"layout-compatible-body");

        let _ = std::fs::remove_dir_all(&mount_root);
    }
}
