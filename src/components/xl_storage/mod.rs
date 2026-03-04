use async_trait::async_trait;
use std::path::PathBuf;
use crate::types::traits::storage_api::StorageAPI;
use crate::types::s3::storage_types::*;
use crate::types::s3::object_layer_types::Context;
use crate::types::errors::StorageError;

pub struct XlStorage {
    path: PathBuf,
}

impl XlStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl StorageAPI for XlStorage {
    async fn make_vol(&self, _ctx: &Context, volume: &str) -> Result<(), StorageError> {
        let vol_path = self.path.join(volume);
        tokio::fs::create_dir_all(&vol_path).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }

    async fn list_vols(&self, _ctx: &Context) -> Result<Vec<VolInfo>, StorageError> {
        let mut vols = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.path).await
            .map_err(|e| StorageError::Io(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| StorageError::Io(e.to_string()))? {
            if entry.file_type().await.map_err(|e| StorageError::Io(e.to_string()))?.is_dir() {
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
            return Err(StorageError::VolumeNotFound(volume.to_string()));
        }
        Ok(VolInfo {
            name: volume.to_string(),
            created: 0,
        })
    }

    async fn delete_vol(&self, _ctx: &Context, volume: &str, _force: bool) -> Result<(), StorageError> {
        let vol_path = self.path.join(volume);
        tokio::fs::remove_dir_all(&vol_path).await
            .map_err(|e| StorageError::Io(e.to_string()))
    }

    async fn read_version(&self, _ctx: &Context, _volume: &str, _path: &str, _version_id: &str) -> Result<FileInfo, StorageError> {
        todo!()
    }

    async fn write_metadata(&self, _ctx: &Context, _volume: &str, _path: &str, _fi: FileInfo) -> Result<(), StorageError> {
        todo!()
    }

    async fn delete_version(&self, _ctx: &Context, _volume: &str, _path: &str, _fi: FileInfo) -> Result<(), StorageError> {
        todo!()
    }

    async fn read_file(&self, _ctx: &Context, _volume: &str, _path: &str, _offset: i64, _buf: &mut [u8]) -> Result<i64, StorageError> {
        todo!()
    }

    async fn create_file(&self, _ctx: &Context, _volume: &str, _path: &str, _size: i64, _reader: crate::types::s3::core::BoxByteStream) -> Result<(), StorageError> {
        todo!()
    }

    async fn append_file(&self, _ctx: &Context, _volume: &str, _path: &str, _buf: &[u8]) -> Result<(), StorageError> {
        todo!()
    }

    async fn rename_file(&self, _ctx: &Context, _src_vol: &str, _src_path: &str, _dst_vol: &str, _dst_path: &str) -> Result<(), StorageError> {
        todo!()
    }
}
