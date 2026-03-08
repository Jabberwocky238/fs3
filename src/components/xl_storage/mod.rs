use async_trait::async_trait;
use std::path::PathBuf;
use crate::types::traits::storage_api::*;
use crate::types::s3::storage_types::*;
use crate::types::s3::object_layer_types::Context;
use crate::types::errors::StorageError;

mod xl_types;
use xl_types::{XlMetaV2, XlMetaV2ShallowVersion, XlMetaV2Object, XlMetaV2VersionHeader};
use xl_types::xl_meta_v2_object::{ErasureAlgo, ChecksumAlgo};

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
        let path = std::env::var("FS3_MOUNT_POINT")
            .unwrap_or_else(|_| ".debug".to_string());
        Self { path: PathBuf::from(path) }
    }

    fn xl_meta_path(&self, volume: &str, path: &str) -> PathBuf {
        self.path.join(volume).join(path).join("xl.meta")
    }

    fn bucket_policy_path(&self, bucket: &str) -> PathBuf {
        self.path.join(bucket).join(".minio.sys").join("policy.json")
    }

    fn bucket_tags_path(&self, bucket: &str) -> PathBuf {
        self.path.join(bucket).join(".minio.sys").join("tags.json")
    }

    fn bucket_versioning_path(&self, bucket: &str) -> PathBuf {
        self.path.join(bucket).join(".minio.sys").join("versioning.json")
    }

    fn bucket_cors_path(&self, bucket: &str) -> PathBuf {
        self.path.join(bucket).join(".minio.sys").join("cors.json")
    }

    fn object_tags_path(&self, bucket: &str, key: &str) -> PathBuf {
        self.path.join(bucket).join(key).join("tags.json")
    }
}

#[async_trait]
impl StorageVolume for XlStorage {
    async fn make_vol(&self, _ctx: &Context, volume: &str) -> Result<(), StorageError> {
        tokio::fs::create_dir_all(self.path.join(volume)).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }

    async fn list_vols(&self, _ctx: &Context) -> Result<Vec<VolInfo>, StorageError> {
        let mut vols = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.path).await
            .map_err(|e| StorageError::Io(e.to_string()))?;
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| StorageError::Io(e.to_string()))? {
            if entry.file_type().await.map_err(|e| StorageError::Io(e.to_string()))?.is_dir() {
                vols.push(VolInfo { name: entry.file_name().to_string_lossy().to_string(), created: 0 });
            }
        }
        Ok(vols)
    }

    async fn stat_vol(&self, _ctx: &Context, volume: &str) -> Result<VolInfo, StorageError> {
        let vol_path = self.path.join(volume);
        if !vol_path.exists() {
            return Err(StorageError::VolumeNotFound(volume.to_string()));
        }
        Ok(VolInfo { name: volume.to_string(), created: 0 })
    }

    async fn delete_vol(&self, _ctx: &Context, volume: &str, _force: bool) -> Result<(), StorageError> {
        tokio::fs::remove_dir_all(self.path.join(volume)).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }
}

#[async_trait]
impl StorageMetadata for XlStorage {
    async fn read_version(&self, _ctx: &Context, volume: &str, path: &str, version_id: &str) -> Result<FileInfo, StorageError> {
        let meta_path = self.xl_meta_path(volume, path);
        let data = tokio::fs::read(&meta_path).await
            .map_err(|_| StorageError::FileNotFound(path.to_string()))?;
        let xl_meta: XlMetaV2 = data.into();

        if xl_meta.versions.is_empty() {
            return Err(StorageError::FileNotFound("no versions".to_string()));
        }

        let version = &xl_meta.versions[0];
        let obj_data = &version.meta;
        let obj: XlMetaV2Object = obj_data.clone().into();

        let vid = uuid::Uuid::from_bytes(obj.version_id).to_string();
        let ddir = uuid::Uuid::from_bytes(obj.data_dir).to_string();

        Ok(FileInfo {
            volume: volume.to_string(),
            name: path.to_string(),
            version_id: vid,
            size: obj.size as u64,
            data_dir: ddir,
            user_metadata: obj.meta_user.clone().unwrap_or_default().into_iter().collect(),
            erasure_index: obj.erasure_index,
            erasure_m: obj.erasure_m,
            erasure_n: obj.erasure_n,
        })
    }

    async fn write_metadata(&self, _ctx: &Context, volume: &str, path: &str, fi: FileInfo) -> Result<(), StorageError> {
        let meta_path = self.xl_meta_path(volume, path);
        if let Some(parent) = meta_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| StorageError::Io(e.to_string()))?;
        }

        let vid = uuid::Uuid::parse_str(&fi.version_id)
            .map_err(|e| StorageError::Io(e.to_string()))?;
        let ddir = uuid::Uuid::parse_str(&fi.data_dir)
            .map_err(|e| StorageError::Io(e.to_string()))?;

        let mut meta_sys = std::collections::HashMap::new();
        let inline_data = if fi.size < 128 * 1024 {
            let data_path = self.path.join(volume).join(path).join(&fi.data_dir);
            let data = tokio::fs::read(&data_path).await.unwrap_or_default();
            meta_sys.insert("x-minio-internal-inline-data".to_string(), b"true".to_vec());
            let _ = tokio::fs::remove_file(&data_path).await;
            data
        } else {
            Vec::new()
        };

        let obj = XlMetaV2Object {
            version_id: *vid.as_bytes(),
            data_dir: *ddir.as_bytes(),
            erasure_algorithm: ErasureAlgo::ReedSolomon,
            erasure_m: fi.erasure_m,
            erasure_n: fi.erasure_n,
            erasure_block_size: 1048576,
            erasure_index: fi.erasure_index,
            erasure_dist: vec![fi.erasure_index as u8],
            bitrot_checksum_algo: ChecksumAlgo::HighwayHash,
            part_numbers: vec![1],
            part_etags: vec![],
            part_sizes: vec![fi.size as i64],
            part_actual_sizes: Some(vec![fi.size as i64]),
            part_indices: Some(vec![]),
            size: fi.size as i64,
            mod_time: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
            meta_sys: Some(meta_sys.into_iter().collect()),
            meta_user: Some(fi.user_metadata.into_iter().collect()),
        };

        let obj_bytes: Vec<u8> = obj.into();
        let header = XlMetaV2VersionHeader {
            version_id: *vid.as_bytes(),
            mod_time: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as i64,
            signature: [0x78, 0x6c, 0x32, 0x20],
            version_type: 1,
            flags: 0,
            ec_n: fi.erasure_n as u8,
            ec_m: fi.erasure_m as u8,
        };

        let xl_meta = XlMetaV2 {
            versions: vec![XlMetaV2ShallowVersion {
                header,
                meta: obj_bytes,
            }],
            inline_data: if inline_data.is_empty() { None } else { Some(inline_data) },
            meta_v: 1,
        };

        let data: Vec<u8> = xl_meta.into();
        tokio::fs::write(&meta_path, &data).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }

    async fn delete_version(&self, _ctx: &Context, volume: &str, path: &str, _fi: FileInfo) -> Result<(), StorageError> {
        tokio::fs::remove_file(self.xl_meta_path(volume, path)).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }
}

#[async_trait]
impl StorageFile for XlStorage {
    async fn read_file(&self, _ctx: &Context, volume: &str, path: &str, offset: i64, buf: &mut [u8]) -> Result<i64, StorageError> {
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
        let mut file = tokio::fs::File::open(&file_path).await
            .map_err(|_| StorageError::FileNotFound(path.to_string()))?;
        use tokio::io::{AsyncReadExt, AsyncSeekExt};
        file.seek(std::io::SeekFrom::Start(offset as u64)).await
            .map_err(|e| StorageError::Io(e.to_string()))?;
        let n = file.read(buf).await
            .map_err(|e| StorageError::Io(e.to_string()))?;
        Ok(n as i64)
    }

    async fn create_file(&self, _ctx: &Context, volume: &str, path: &str, _size: i64, mut reader: crate::types::s3::core::BoxByteStream) -> Result<u64, StorageError> {
        let file_path = self.path.join(volume).join(path);
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| StorageError::Io(e.to_string()))?;
        }
        let mut file = tokio::fs::File::create(&file_path).await
            .map_err(|e| StorageError::Io(e.to_string()))?;
        use tokio::io::AsyncWriteExt;
        use futures::StreamExt;
        let mut total = 0u64;
        while let Some(chunk) = reader.next().await {
            let bytes = chunk.map_err(|e| StorageError::Io(e.to_string()))?;
            file.write_all(&bytes).await
                .map_err(|e| StorageError::Io(e.to_string()))?;
            total += bytes.len() as u64;
        }
        Ok(total)
    }

    async fn append_file(&self, _ctx: &Context, volume: &str, path: &str, buf: &[u8]) -> Result<(), StorageError> {
        let file_path = self.path.join(volume).join(path);
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(&file_path).await
            .map_err(|e| StorageError::Io(e.to_string()))?;
        file.write_all(buf).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }

    async fn rename_file(&self, _ctx: &Context, src_vol: &str, src_path: &str, dst_vol: &str, dst_path: &str) -> Result<(), StorageError> {
        let src = self.path.join(src_vol).join(src_path);
        let dst = self.path.join(dst_vol).join(dst_path);
        tokio::fs::rename(&src, &dst).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }
}

#[async_trait]
impl StorageBucketConfig for XlStorage {
    async fn read_bucket_policy(&self, _ctx: &Context, bucket: &str) -> Result<Option<String>, StorageError> {
        match tokio::fs::read_to_string(self.bucket_policy_path(bucket)).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }

    async fn write_bucket_policy(&self, _ctx: &Context, bucket: &str, policy: &str) -> Result<(), StorageError> {
        let path = self.bucket_policy_path(bucket);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| StorageError::Io(e.to_string()))?;
        }
        tokio::fs::write(&path, policy).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }

    async fn delete_bucket_policy(&self, _ctx: &Context, bucket: &str) -> Result<(), StorageError> {
        match tokio::fs::remove_file(self.bucket_policy_path(bucket)).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }

    async fn read_bucket_tags(&self, _ctx: &Context, bucket: &str) -> Result<Option<String>, StorageError> {
        match tokio::fs::read_to_string(self.bucket_tags_path(bucket)).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }

    async fn write_bucket_tags(&self, _ctx: &Context, bucket: &str, tags: &str) -> Result<(), StorageError> {
        let path = self.bucket_tags_path(bucket);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| StorageError::Io(e.to_string()))?;
        }
        tokio::fs::write(&path, tags).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }

    async fn delete_bucket_tags(&self, _ctx: &Context, bucket: &str) -> Result<(), StorageError> {
        match tokio::fs::remove_file(self.bucket_tags_path(bucket)).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }

    async fn read_bucket_versioning(&self, _ctx: &Context, bucket: &str) -> Result<Option<String>, StorageError> {
        match tokio::fs::read_to_string(self.bucket_versioning_path(bucket)).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }

    async fn write_bucket_versioning(&self, _ctx: &Context, bucket: &str, status: &str) -> Result<(), StorageError> {
        let path = self.bucket_versioning_path(bucket);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| StorageError::Io(e.to_string()))?;
        }
        tokio::fs::write(&path, status).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }

    async fn read_bucket_cors(&self, _ctx: &Context, bucket: &str) -> Result<Option<String>, StorageError> {
        match tokio::fs::read_to_string(self.bucket_cors_path(bucket)).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }

    async fn write_bucket_cors(&self, _ctx: &Context, bucket: &str, cors: &str) -> Result<(), StorageError> {
        let path = self.bucket_cors_path(bucket);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| StorageError::Io(e.to_string()))?;
        }
        tokio::fs::write(&path, cors).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }

    async fn delete_bucket_cors(&self, _ctx: &Context, bucket: &str) -> Result<(), StorageError> {
        match tokio::fs::remove_file(self.bucket_cors_path(bucket)).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }
}

#[async_trait]
impl StorageObjectConfig for XlStorage {
    async fn read_object_tags(&self, _ctx: &Context, bucket: &str, key: &str) -> Result<Option<String>, StorageError> {
        match tokio::fs::read_to_string(self.object_tags_path(bucket, key)).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }

    async fn write_object_tags(&self, _ctx: &Context, bucket: &str, key: &str, tags: &str) -> Result<(), StorageError> {
        let path = self.object_tags_path(bucket, key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| StorageError::Io(e.to_string()))?;
        }
        tokio::fs::write(&path, tags).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }

    async fn delete_object_tags(&self, _ctx: &Context, bucket: &str, key: &str) -> Result<(), StorageError> {
        match tokio::fs::remove_file(self.object_tags_path(bucket, key)).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }
}
