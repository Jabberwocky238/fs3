use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;

use crate::components::remote_storage::wire::{
    MinioDeleteFileRequest, MinioStorageCall, MinioWriteAllRequest,
    STORAGE_REST_METHOD_READ_VERSION, STORAGE_REST_PARAM_DISK_ID, STORAGE_REST_PARAM_FILE_PATH,
    STORAGE_REST_PARAM_HEALING, STORAGE_REST_PARAM_INCLUDE_FREE_VERSIONS,
    STORAGE_REST_PARAM_ORIG_VOLUME, STORAGE_REST_PARAM_VERSION_ID, STORAGE_REST_PARAM_VOLUME,
    STORAGE_REST_VERSION, decode_minio_file_info, encode_minio_delete_file_request,
    encode_minio_write_all_request,
};
use crate::types::FS3Error;
use crate::types::s3::core::BoxByteStream;
use crate::types::s3::object_layer_types::Context;
use crate::types::s3::storage_types::{
    CreateFileOptions, DeletePathOptions, FileInfo, RenameDataOptions, RenameDataResult, VolInfo,
    WriteAllOptions,
};
use crate::types::storage_endpoint::StorageEndpoint;
use crate::types::traits::storage_api::{
    StorageBucketConfig, StorageFile, StorageMetadata, StorageObjectConfig, StorageVolume,
};

#[derive(Debug, Clone)]
pub struct RemoteStorageClient {
    endpoint: StorageEndpoint,
    http: Client,
}

impl RemoteStorageClient {
    pub fn new(endpoint: StorageEndpoint) -> Self {
        Self {
            endpoint,
            http: Client::new(),
        }
    }

    pub fn endpoint(&self) -> &StorageEndpoint {
        &self.endpoint
    }

    pub fn storage_id(&self) -> &str {
        &self.endpoint.storage_id
    }

    fn disk_id(&self) -> &str {
        &self.endpoint.storage_id
    }

    fn unsupported(&self, method: &str) -> FS3Error {
        FS3Error::internal(format!(
            "remote storage method {method} is not implemented for endpoint {}",
            self.endpoint.address
        ))
    }

    fn minio_transport_unimplemented(&self, call: MinioStorageCall) -> FS3Error {
        FS3Error::internal(format!(
            "MinIO-compatible remote storage transport for {} is not implemented for endpoint {}",
            call.as_str(),
            self.endpoint.address,
        ))
    }

    pub fn storage_rest_url(&self) -> String {
        format!(
            "{}/minio/storage/{}/{}",
            self.endpoint.address.trim_end_matches('/'),
            self.endpoint.storage_id,
            STORAGE_REST_VERSION,
        )
    }

    fn storage_rest_method_url(&self, method: &str) -> String {
        format!("{}{}", self.storage_rest_url(), method)
    }

    fn auth_token(&self) -> Option<String> {
        std::env::var("FS3_REMOTE_AUTH_TOKEN").ok()
    }
}

#[async_trait]
impl StorageVolume<FS3Error> for RemoteStorageClient {
    async fn make_vol(&self, _ctx: &Context, _volume: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("make_vol"))
    }

    async fn list_vols(&self, _ctx: &Context) -> Result<Vec<VolInfo>, FS3Error> {
        Err(self.unsupported("list_vols"))
    }

    async fn stat_vol(&self, _ctx: &Context, _volume: &str) -> Result<VolInfo, FS3Error> {
        Err(self.unsupported("stat_vol"))
    }

    async fn delete_vol(
        &self,
        _ctx: &Context,
        _volume: &str,
        _force: bool,
    ) -> Result<(), FS3Error> {
        Err(self.unsupported("delete_vol"))
    }
}

#[async_trait]
impl StorageMetadata<FS3Error> for RemoteStorageClient {
    async fn read_version(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        version_id: &str,
    ) -> Result<FileInfo, FS3Error> {
        let mut request = self
            .http
            .get(self.storage_rest_method_url(STORAGE_REST_METHOD_READ_VERSION))
            .query(&[
                (STORAGE_REST_PARAM_DISK_ID, self.disk_id()),
                (STORAGE_REST_PARAM_ORIG_VOLUME, ""),
                (STORAGE_REST_PARAM_VOLUME, volume),
                (STORAGE_REST_PARAM_FILE_PATH, path),
                (STORAGE_REST_PARAM_VERSION_ID, version_id),
                (STORAGE_REST_PARAM_INCLUDE_FREE_VERSIONS, "false"),
                (STORAGE_REST_PARAM_HEALING, "false"),
            ]);

        if let Some(token) = self.auth_token() {
            request = request
                .header("Authorization", format!("Bearer {token}"))
                .header(
                    "X-Minio-Time",
                    Utc::now()
                        .timestamp_nanos_opt()
                        .unwrap_or_default()
                        .to_string(),
                );
        }

        let response = request.send().await.map_err(|err| {
            FS3Error::internal(format!("MinIO ReadVersion request failed: {err}"))
        })?;
        let status = response.status();
        let body = response.bytes().await.map_err(|err| {
            FS3Error::internal(format!("MinIO ReadVersion response read failed: {err}"))
        })?;

        if !status.is_success() {
            let message = String::from_utf8_lossy(&body).trim().to_string();
            return Err(FS3Error::new(
                status,
                if message.is_empty() {
                    format!("MinIO ReadVersion request failed with status {status}")
                } else {
                    message
                },
            ));
        }

        decode_minio_file_info(&body)
            .map_err(|err| FS3Error::internal(format!("MinIO ReadVersion decode failed: {err}")))
    }

    async fn write_all(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        data: &[u8],
        _opts: WriteAllOptions,
    ) -> Result<(), FS3Error> {
        let _payload = encode_minio_write_all_request(&MinioWriteAllRequest {
            disk_id: self.disk_id().to_string(),
            volume: volume.to_string(),
            file_path: path.to_string(),
            buf: data.to_vec(),
        })
        .map_err(|err| {
            FS3Error::internal(format!("MinIO WriteAll payload encode failed: {err}"))
        })?;
        Err(self.minio_transport_unimplemented(MinioStorageCall::WriteAll))
    }

    async fn write_metadata(
        &self,
        _ctx: &Context,
        _volume: &str,
        _path: &str,
        _fi: FileInfo,
    ) -> Result<(), FS3Error> {
        Err(self.unsupported("write_metadata"))
    }

    async fn rename_data(
        &self,
        _ctx: &Context,
        _src_volume: &str,
        _src_path: &str,
        _fi: FileInfo,
        _dst_volume: &str,
        _dst_path: &str,
        _opts: RenameDataOptions,
    ) -> Result<RenameDataResult, FS3Error> {
        Err(self.minio_transport_unimplemented(MinioStorageCall::RenameData))
    }

    async fn delete_version(
        &self,
        _ctx: &Context,
        _volume: &str,
        _path: &str,
        _fi: FileInfo,
    ) -> Result<(), FS3Error> {
        Err(self.unsupported("delete_version"))
    }
}

#[async_trait]
impl StorageFile<FS3Error> for RemoteStorageClient {
    async fn read_file(
        &self,
        _ctx: &Context,
        _volume: &str,
        _path: &str,
        _offset: i64,
        _buf: &mut [u8],
    ) -> Result<i64, FS3Error> {
        Err(self.unsupported("read_file"))
    }

    async fn create_file(
        &self,
        _ctx: &Context,
        _volume: &str,
        _path: &str,
        _size: i64,
        _reader: BoxByteStream,
        _opts: CreateFileOptions,
    ) -> Result<u64, FS3Error> {
        Err(self.unsupported("create_file"))
    }

    async fn append_file(
        &self,
        _ctx: &Context,
        _volume: &str,
        _path: &str,
        _buf: &[u8],
    ) -> Result<(), FS3Error> {
        Err(self.unsupported("append_file"))
    }

    async fn rename_file(
        &self,
        _ctx: &Context,
        _src_vol: &str,
        _src_path: &str,
        _dst_vol: &str,
        _dst_path: &str,
    ) -> Result<(), FS3Error> {
        Err(self.unsupported("rename_file"))
    }

    async fn delete_path(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        opts: DeletePathOptions,
    ) -> Result<(), FS3Error> {
        let _payload = encode_minio_delete_file_request(&MinioDeleteFileRequest {
            disk_id: self.disk_id().to_string(),
            volume: volume.to_string(),
            file_path: path.to_string(),
            opts: opts.into(),
        })
        .map_err(|err| FS3Error::internal(format!("MinIO Delete payload encode failed: {err}")))?;
        Err(self.minio_transport_unimplemented(MinioStorageCall::Delete))
    }
}

#[async_trait]
impl StorageBucketConfig<FS3Error> for RemoteStorageClient {
    async fn read_bucket_policy(
        &self,
        _ctx: &Context,
        _bucket: &str,
    ) -> Result<Option<String>, FS3Error> {
        Err(self.unsupported("read_bucket_policy"))
    }

    async fn write_bucket_policy(
        &self,
        _ctx: &Context,
        _bucket: &str,
        _policy: &str,
    ) -> Result<(), FS3Error> {
        Err(self.unsupported("write_bucket_policy"))
    }

    async fn delete_bucket_policy(&self, _ctx: &Context, _bucket: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("delete_bucket_policy"))
    }

    async fn read_bucket_tags(
        &self,
        _ctx: &Context,
        _bucket: &str,
    ) -> Result<Option<String>, FS3Error> {
        Err(self.unsupported("read_bucket_tags"))
    }

    async fn write_bucket_tags(
        &self,
        _ctx: &Context,
        _bucket: &str,
        _tags: &str,
    ) -> Result<(), FS3Error> {
        Err(self.unsupported("write_bucket_tags"))
    }

    async fn delete_bucket_tags(&self, _ctx: &Context, _bucket: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("delete_bucket_tags"))
    }

    async fn read_bucket_versioning(
        &self,
        _ctx: &Context,
        _bucket: &str,
    ) -> Result<Option<String>, FS3Error> {
        Err(self.unsupported("read_bucket_versioning"))
    }

    async fn write_bucket_versioning(
        &self,
        _ctx: &Context,
        _bucket: &str,
        _status: &str,
    ) -> Result<(), FS3Error> {
        Err(self.unsupported("write_bucket_versioning"))
    }

    async fn read_bucket_cors(
        &self,
        _ctx: &Context,
        _bucket: &str,
    ) -> Result<Option<String>, FS3Error> {
        Err(self.unsupported("read_bucket_cors"))
    }

    async fn write_bucket_cors(
        &self,
        _ctx: &Context,
        _bucket: &str,
        _cors: &str,
    ) -> Result<(), FS3Error> {
        Err(self.unsupported("write_bucket_cors"))
    }

    async fn delete_bucket_cors(&self, _ctx: &Context, _bucket: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("delete_bucket_cors"))
    }
}

#[async_trait]
impl StorageObjectConfig<FS3Error> for RemoteStorageClient {
    async fn read_object_tags(
        &self,
        _ctx: &Context,
        _bucket: &str,
        _key: &str,
    ) -> Result<Option<String>, FS3Error> {
        Err(self.unsupported("read_object_tags"))
    }

    async fn write_object_tags(
        &self,
        _ctx: &Context,
        _bucket: &str,
        _key: &str,
        _tags: &str,
    ) -> Result<(), FS3Error> {
        Err(self.unsupported("write_object_tags"))
    }

    async fn delete_object_tags(
        &self,
        _ctx: &Context,
        _bucket: &str,
        _key: &str,
    ) -> Result<(), FS3Error> {
        Err(self.unsupported("delete_object_tags"))
    }
}
