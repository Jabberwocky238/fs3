use crate::types::errors::StorageError;
use crate::types::s3::object_layer_types::Context;
use crate::types::s3::storage_types::*;
use crate::types::traits::storage_api::*;
use async_trait::async_trait;
use std::path::PathBuf;

mod xl_types;
use xl_types::xl_meta_v2_object::{ChecksumAlgo, ErasureAlgo};
use xl_types::xl_meta_v2_version::VersionType;
use xl_types::{
    XlMetaV2, XlMetaV2Object, XlMetaV2ShallowVersion, XlMetaV2Version, XlMetaV2VersionHeader,
};

pub struct XlStorage {
    path: PathBuf,
}

impl XlStorage {
    pub fn new(path: PathBuf) -> Self {
        let _ = Self::initialize(&path);
        Self { path }
    }

    fn initialize(path: &PathBuf) -> std::io::Result<()> {
        use std::fs;
        let sys = path.join(".minio.sys");
        fs::create_dir_all(sys.join("buckets"))?;
        fs::create_dir_all(sys.join("config/iam"))?;
        fs::create_dir_all(sys.join("multipart"))?;
        fs::create_dir_all(sys.join("tmp/.trash"))?;

        // Create format.json
        let format_path = sys.join("format.json");
        if !format_path.exists() {
            use uuid::Uuid;
            let pool_id = Uuid::new_v4();
            let disk_id = Uuid::new_v4();
            let format = format!(
                r#"{{"version":"1","format":"xl-single","id":"{}","xl":{{"version":"3","this":"{}","sets":[["{}"]],"distributionAlgo":"SIPMOD+PARITY"}}}}"#,
                pool_id, disk_id, disk_id
            );
            fs::write(format_path, format)?;
        }

        // Create IAM format.json
        let iam_format_path = sys.join("config/iam/format.json");
        if !iam_format_path.exists() {
            fs::write(iam_format_path, r#"{"version":1}"#)?;
        }

        // Create placeholder directories for xl.meta files
        fs::create_dir_all(sys.join("pool.bin"))?;
        fs::create_dir_all(sys.join("config/config.json"))?;
        fs::create_dir_all(sys.join("buckets/.bloomcycle.bin"))?;
        fs::create_dir_all(sys.join("buckets/.usage.json"))?;

        Ok(())
    }

    pub fn from_env() -> Self {
        let path = std::env::var("FS3_MOUNT_POINT").unwrap_or_else(|_| ".debug".to_string());
        Self {
            path: PathBuf::from(path),
        }
    }

    fn xl_meta_path(&self, volume: &str, path: &str) -> PathBuf {
        self.path.join(volume).join(path).join("xl.meta")
    }

    fn bucket_policy_path(&self, bucket: &str) -> PathBuf {
        self.path
            .join(bucket)
            .join(".minio.sys")
            .join("policy.json")
    }

    fn bucket_tags_path(&self, bucket: &str) -> PathBuf {
        self.path.join(bucket).join(".minio.sys").join("tags.json")
    }

    fn bucket_versioning_path(&self, bucket: &str) -> PathBuf {
        self.path
            .join(bucket)
            .join(".minio.sys")
            .join("versioning.json")
    }

    fn bucket_cors_path(&self, bucket: &str) -> PathBuf {
        self.path.join(bucket).join(".minio.sys").join("cors.json")
    }

    fn object_tags_path(&self, bucket: &str, key: &str) -> PathBuf {
        self.path.join(bucket).join(key).join("tags.json")
    }

    fn object_data_path(&self, volume: &str, path: &str, data_dir: &str) -> PathBuf {
        self.path.join(volume).join(path).join(data_dir)
    }

    fn encode_shallow_version(
        &self,
        fi: &FileInfo,
        has_inline_data: bool,
    ) -> Result<XlMetaV2ShallowVersion, StorageError> {
        let vid =
            uuid::Uuid::parse_str(&fi.version_id).map_err(|e| StorageError::from(e.to_string()))?;
        let ddir =
            uuid::Uuid::parse_str(&fi.data_dir).map_err(|e| StorageError::from(e.to_string()))?;
        let mod_time = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);

        let mut meta_sys = std::collections::HashMap::new();
        if has_inline_data {
            meta_sys.insert("x-minio-internal-inline-data".to_string(), b"true".to_vec());
        }
        let erasure_width = std::cmp::max(1, fi.erasure_m + fi.erasure_n) as u8;
        let erasure_dist = (1..=erasure_width).collect::<Vec<_>>();

        let obj = XlMetaV2Object {
            version_id: *vid.as_bytes(),
            data_dir: *ddir.as_bytes(),
            erasure_algorithm: ErasureAlgo::ReedSolomon,
            erasure_m: fi.erasure_m,
            erasure_n: fi.erasure_n,
            erasure_block_size: 1048576,
            erasure_index: fi.erasure_index,
            erasure_dist,
            bitrot_checksum_algo: ChecksumAlgo::HighwayHash,
            part_numbers: vec![1],
            part_etags: vec![],
            part_sizes: vec![fi.size as i64],
            part_actual_sizes: Some(vec![fi.size as i64]),
            part_indices: None,
            size: fi.size as i64,
            mod_time,
            meta_sys: Some(meta_sys.into_iter().collect()),
            meta_user: Some(fi.user_metadata.clone().into_iter().collect()),
        };
        let signature = obj.signature();
        let mut flags = 0u8;
        if obj.uses_data_dir() {
            flags |= 1 << 1;
        }
        if obj.inline_data() {
            flags |= 1 << 2;
        }

        let version = XlMetaV2Version {
            version_type: VersionType::Object,
            object_v1: None,
            object_v2: Some(obj),
            delete_marker: None,
            written_by_version: 0,
        };
        let version_bytes: Vec<u8> = (&version).into();

        let header = XlMetaV2VersionHeader {
            version_id: *vid.as_bytes(),
            mod_time,
            signature,
            version_type: 1,
            flags,
            ec_n: fi.erasure_n as u8,
            ec_m: fi.erasure_m as u8,
        };

        Ok(XlMetaV2ShallowVersion {
            header,
            meta: version_bytes,
        })
    }

    fn encode_metadata(
        &self,
        fi: &FileInfo,
        inline_data: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, StorageError> {
        let xl_meta = XlMetaV2 {
            versions: vec![self.encode_shallow_version(fi, inline_data.is_some())?],
            inline_data,
            meta_v: 1,
        };
        Ok(xl_meta.into())
    }

    fn decode_file_info(
        &self,
        volume: &str,
        path: &str,
        version: &XlMetaV2ShallowVersion,
    ) -> Result<FileInfo, StorageError> {
        let version_body: XlMetaV2Version = version.meta.clone().into();
        let obj = version_body.object_v2.ok_or_else(|| {
            StorageError::from("xl.meta version does not contain object metadata")
        })?;

        Ok(FileInfo {
            volume: volume.to_string(),
            name: path.to_string(),
            version_id: uuid::Uuid::from_bytes(obj.version_id).to_string(),
            size: obj.size as u64,
            data_dir: uuid::Uuid::from_bytes(obj.data_dir).to_string(),
            etag: String::new(),
            content_type: "application/octet-stream".to_string(),
            user_metadata: obj.meta_user.unwrap_or_default().into_iter().collect(),
            erasure_index: obj.erasure_index,
            erasure_m: obj.erasure_m,
            erasure_n: obj.erasure_n,
        })
    }

    async fn read_xl_meta(
        &self,
        volume: &str,
        path: &str,
    ) -> Result<Option<XlMetaV2>, StorageError> {
        match tokio::fs::read(self.xl_meta_path(volume, path)).await {
            Ok(data) => Ok(Some(data.into())),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(StorageError::from(err.to_string())),
        }
    }
}

#[async_trait]
impl StorageVolume<StorageError> for XlStorage {
    async fn make_vol(&self, _ctx: &Context, volume: &str) -> Result<(), StorageError> {
        tokio::fs::create_dir_all(self.path.join(volume))
            .await
            .map_err(|e| StorageError::from(e.to_string()))
    }

    async fn list_vols(&self, _ctx: &Context) -> Result<Vec<VolInfo>, StorageError> {
        let mut vols = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.path)
            .await
            .map_err(|e| StorageError::from(e.to_string()))?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StorageError::from(e.to_string()))?
        {
            if entry
                .file_type()
                .await
                .map_err(|e| StorageError::from(e.to_string()))?
                .is_dir()
            {
                vols.push(VolInfo {
                    name: entry.file_name().to_string_lossy().to_string(),
                    created: 0,
                });
            }
        }
        Ok(vols)
    }

    async fn stat_vol(&self, _ctx: &Context, volume: &str) -> Result<VolInfo, StorageError> {
        let vol_path = self.path.join(volume);
        if !vol_path.exists() {
            return Err(StorageError::from(format!("volume not found: {volume}")));
        }
        Ok(VolInfo {
            name: volume.to_string(),
            created: 0,
        })
    }

    async fn delete_vol(
        &self,
        _ctx: &Context,
        volume: &str,
        _force: bool,
    ) -> Result<(), StorageError> {
        tokio::fs::remove_dir_all(self.path.join(volume))
            .await
            .map_err(|e| StorageError::from(e.to_string()))
    }
}

#[async_trait]
impl StorageMetadata<StorageError> for XlStorage {
    async fn read_version(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        version_id: &str,
    ) -> Result<FileInfo, StorageError> {
        let xl_meta = self
            .read_xl_meta(volume, path)
            .await?
            .ok_or_else(|| StorageError::from(format!("file not found: {path}")))?;

        if xl_meta.versions.is_empty() {
            return Err(StorageError::from("file not found: no versions"));
        }

        let version = if version_id == "null" {
            xl_meta.versions.first()
        } else {
            xl_meta.versions.iter().find(|version| {
                uuid::Uuid::from_bytes(version.header.version_id).to_string() == version_id
            })
        }
        .ok_or_else(|| StorageError::from(format!("file not found: {version_id}")))?;

        self.decode_file_info(volume, path, version)
    }

    async fn write_all(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        data: &[u8],
        _opts: WriteAllOptions,
    ) -> Result<(), StorageError> {
        let file_path = self.path.join(volume).join(path);
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::from(e.to_string()))?;
        }
        tokio::fs::write(&file_path, data)
            .await
            .map_err(|e| StorageError::from(e.to_string()))
    }

    async fn write_metadata(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        fi: FileInfo,
    ) -> Result<(), StorageError> {
        let mut xl_meta = self.read_xl_meta(volume, path).await?.unwrap_or(XlMetaV2 {
            versions: Vec::new(),
            inline_data: None,
            meta_v: 1,
        });
        xl_meta
            .versions
            .insert(0, self.encode_shallow_version(&fi, false)?);
        let data: Vec<u8> = xl_meta.into();
        self.write_all(
            _ctx,
            volume,
            &format!("{path}/xl.meta"),
            &data,
            WriteAllOptions {
                path_kind: StoragePathKind::Final,
                write_kind: StorageWriteKind::Metadata,
            },
        )
        .await
    }

    async fn rename_data(
        &self,
        ctx: &Context,
        src_volume: &str,
        src_path: &str,
        fi: FileInfo,
        dst_volume: &str,
        dst_path: &str,
        opts: RenameDataOptions,
    ) -> Result<RenameDataResult, StorageError> {
        let mut dst_meta = self
            .read_xl_meta(dst_volume, dst_path)
            .await?
            .unwrap_or(XlMetaV2 {
                versions: Vec::new(),
                inline_data: None,
                meta_v: 1,
            });
        let old_data_dir = dst_meta
            .versions
            .first()
            .and_then(|version| self.decode_file_info(dst_volume, dst_path, version).ok())
            .map(|file_info| file_info.data_dir);

        dst_meta
            .versions
            .insert(0, self.encode_shallow_version(&fi, false)?);
        let meta_bytes: Vec<u8> = dst_meta.into();
        self.write_all(
            ctx,
            src_volume,
            &format!("{src_path}/xl.meta"),
            &meta_bytes,
            WriteAllOptions {
                path_kind: opts.path_kind,
                write_kind: StorageWriteKind::Metadata,
            },
        )
        .await?;

        let src_data_path = self.object_data_path(src_volume, src_path, &fi.data_dir);
        let dst_data_path = self.object_data_path(dst_volume, dst_path, &fi.data_dir);
        if let Some(parent) = dst_data_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::from(e.to_string()))?;
        }
        tokio::fs::rename(&src_data_path, &dst_data_path)
            .await
            .map_err(|e| StorageError::from(e.to_string()))?;

        let src_meta_path = self.xl_meta_path(src_volume, src_path);
        let dst_meta_path = self.xl_meta_path(dst_volume, dst_path);
        if let Some(parent) = dst_meta_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::from(e.to_string()))?;
        }
        tokio::fs::rename(&src_meta_path, &dst_meta_path)
            .await
            .map_err(|e| StorageError::from(e.to_string()))?;

        let old_data_path = old_data_dir.as_ref().and_then(|dir| {
            if dir == &fi.data_dir {
                None
            } else {
                Some(format!("{dst_path}/{dir}"))
            }
        });

        Ok(RenameDataResult {
            old_data_dir,
            old_data_path,
            cleanup_src_volume: if opts.defer_src_path_cleanup {
                src_volume.to_string()
            } else {
                String::new()
            },
            cleanup_src_path: if opts.defer_src_path_cleanup {
                src_path.to_string()
            } else {
                String::new()
            },
        })
    }

    async fn delete_version(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        _fi: FileInfo,
    ) -> Result<(), StorageError> {
        tokio::fs::remove_file(self.xl_meta_path(volume, path))
            .await
            .map_err(|e| StorageError::from(e.to_string()))
    }
}

#[async_trait]
impl StorageFile<StorageError> for XlStorage {
    async fn read_file(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        offset: i64,
        buf: &mut [u8],
    ) -> Result<i64, StorageError> {
        // Check if data is inline in xl.meta
        let parts: Vec<&str> = path.rsplitn(2, '/').collect();
        if parts.len() == 2 {
            let obj_path = parts[1];
            let meta_path = self.xl_meta_path(volume, obj_path);
            if let Ok(meta_data) = tokio::fs::read(&meta_path).await {
                let xl_meta: XlMetaV2 = meta_data.into();
                if let Some(ref data) = xl_meta.inline_data {
                    let start = offset as usize;
                    let end = std::cmp::min(start + buf.len(), data.len());
                    if start < data.len() {
                        let n = end - start;
                        buf[..n].copy_from_slice(&data[start..end]);
                        return Ok(n as i64);
                    }
                    return Ok(0);
                }
            }
        }

        // Fallback to regular file read
        let file_path = self.path.join(volume).join(path);
        let mut file = tokio::fs::File::open(&file_path)
            .await
            .map_err(|_| StorageError::from(format!("file not found: {path}")))?;
        use tokio::io::{AsyncReadExt, AsyncSeekExt};
        file.seek(std::io::SeekFrom::Start(offset as u64))
            .await
            .map_err(|e| StorageError::from(e.to_string()))?;
        let n = file
            .read(buf)
            .await
            .map_err(|e| StorageError::from(e.to_string()))?;
        Ok(n as i64)
    }

    async fn create_file(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        _size: i64,
        mut reader: crate::types::s3::core::BoxByteStream,
        _opts: CreateFileOptions,
    ) -> Result<u64, StorageError> {
        let file_path = self.path.join(volume).join(path);
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::from(e.to_string()))?;
        }
        let mut file = tokio::fs::File::create(&file_path)
            .await
            .map_err(|e| StorageError::from(e.to_string()))?;
        use futures::StreamExt;
        use tokio::io::AsyncWriteExt;
        let mut total = 0u64;
        while let Some(chunk) = reader.next().await {
            let bytes = chunk.map_err(|e| StorageError::from(e.to_string()))?;
            file.write_all(&bytes)
                .await
                .map_err(|e| StorageError::from(e.to_string()))?;
            total += bytes.len() as u64;
        }
        Ok(total)
    }

    async fn append_file(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        buf: &[u8],
    ) -> Result<(), StorageError> {
        let file_path = self.path.join(volume).join(path);
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(&file_path)
            .await
            .map_err(|e| StorageError::from(e.to_string()))?;
        file.write_all(buf)
            .await
            .map_err(|e| StorageError::from(e.to_string()))
    }

    async fn rename_file(
        &self,
        _ctx: &Context,
        src_vol: &str,
        src_path: &str,
        dst_vol: &str,
        dst_path: &str,
    ) -> Result<(), StorageError> {
        let src = self.path.join(src_vol).join(src_path);
        let dst = self.path.join(dst_vol).join(dst_path);
        tokio::fs::rename(&src, &dst)
            .await
            .map_err(|e| StorageError::from(e.to_string()))
    }

    async fn delete_path(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        opts: DeletePathOptions,
    ) -> Result<(), StorageError> {
        let target = self.path.join(volume).join(path);
        let result = if opts.recursive {
            tokio::fs::remove_dir_all(&target).await
        } else {
            tokio::fs::remove_file(&target).await
        };
        match result {
            Ok(_) => Ok(()),
            Err(err) if opts.ignore_not_found && err.kind() == std::io::ErrorKind::NotFound => {
                Ok(())
            }
            Err(err) => Err(StorageError::from(err.to_string())),
        }
    }
}

#[async_trait]
impl StorageBucketConfig<StorageError> for XlStorage {
    async fn read_bucket_policy(
        &self,
        _ctx: &Context,
        bucket: &str,
    ) -> Result<Option<String>, StorageError> {
        match tokio::fs::read_to_string(self.bucket_policy_path(bucket)).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::from(e.to_string())),
        }
    }

    async fn write_bucket_policy(
        &self,
        _ctx: &Context,
        bucket: &str,
        policy: &str,
    ) -> Result<(), StorageError> {
        let path = self.bucket_policy_path(bucket);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::from(e.to_string()))?;
        }
        tokio::fs::write(&path, policy)
            .await
            .map_err(|e| StorageError::from(e.to_string()))
    }

    async fn delete_bucket_policy(&self, _ctx: &Context, bucket: &str) -> Result<(), StorageError> {
        match tokio::fs::remove_file(self.bucket_policy_path(bucket)).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::from(e.to_string())),
        }
    }

    async fn read_bucket_tags(
        &self,
        _ctx: &Context,
        bucket: &str,
    ) -> Result<Option<String>, StorageError> {
        match tokio::fs::read_to_string(self.bucket_tags_path(bucket)).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::from(e.to_string())),
        }
    }

    async fn write_bucket_tags(
        &self,
        _ctx: &Context,
        bucket: &str,
        tags: &str,
    ) -> Result<(), StorageError> {
        let path = self.bucket_tags_path(bucket);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::from(e.to_string()))?;
        }
        tokio::fs::write(&path, tags)
            .await
            .map_err(|e| StorageError::from(e.to_string()))
    }

    async fn delete_bucket_tags(&self, _ctx: &Context, bucket: &str) -> Result<(), StorageError> {
        match tokio::fs::remove_file(self.bucket_tags_path(bucket)).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::from(e.to_string())),
        }
    }

    async fn read_bucket_versioning(
        &self,
        _ctx: &Context,
        bucket: &str,
    ) -> Result<Option<String>, StorageError> {
        match tokio::fs::read_to_string(self.bucket_versioning_path(bucket)).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::from(e.to_string())),
        }
    }

    async fn write_bucket_versioning(
        &self,
        _ctx: &Context,
        bucket: &str,
        status: &str,
    ) -> Result<(), StorageError> {
        let path = self.bucket_versioning_path(bucket);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::from(e.to_string()))?;
        }
        tokio::fs::write(&path, status)
            .await
            .map_err(|e| StorageError::from(e.to_string()))
    }

    async fn read_bucket_cors(
        &self,
        _ctx: &Context,
        bucket: &str,
    ) -> Result<Option<String>, StorageError> {
        match tokio::fs::read_to_string(self.bucket_cors_path(bucket)).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::from(e.to_string())),
        }
    }

    async fn write_bucket_cors(
        &self,
        _ctx: &Context,
        bucket: &str,
        cors: &str,
    ) -> Result<(), StorageError> {
        let path = self.bucket_cors_path(bucket);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::from(e.to_string()))?;
        }
        tokio::fs::write(&path, cors)
            .await
            .map_err(|e| StorageError::from(e.to_string()))
    }

    async fn delete_bucket_cors(&self, _ctx: &Context, bucket: &str) -> Result<(), StorageError> {
        match tokio::fs::remove_file(self.bucket_cors_path(bucket)).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::from(e.to_string())),
        }
    }
}

#[async_trait]
impl StorageObjectConfig<StorageError> for XlStorage {
    async fn read_object_tags(
        &self,
        _ctx: &Context,
        bucket: &str,
        key: &str,
    ) -> Result<Option<String>, StorageError> {
        match tokio::fs::read_to_string(self.object_tags_path(bucket, key)).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::from(e.to_string())),
        }
    }

    async fn write_object_tags(
        &self,
        _ctx: &Context,
        bucket: &str,
        key: &str,
        tags: &str,
    ) -> Result<(), StorageError> {
        let path = self.object_tags_path(bucket, key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::from(e.to_string()))?;
        }
        tokio::fs::write(&path, tags)
            .await
            .map_err(|e| StorageError::from(e.to_string()))
    }

    async fn delete_object_tags(
        &self,
        _ctx: &Context,
        bucket: &str,
        key: &str,
    ) -> Result<(), StorageError> {
        match tokio::fs::remove_file(self.object_tags_path(bucket, key)).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::from(e.to_string())),
        }
    }
}
