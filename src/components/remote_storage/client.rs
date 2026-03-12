use async_trait::async_trait;
use reqwest::Client;

use crate::types::FS3Error;
use crate::types::s3::core::BoxByteStream;
use crate::types::s3::object_layer_types::Context;
use crate::types::s3::storage_types::{
    CreateFileOptions, DeletePathOptions, FileInfo, RenameDataOptions, RenameDataResult, VolInfo,
    WriteAllOptions,
};
use crate::types::storage_endpoint::StorageEndpoint;
use crate::types::storage_remote::{
    DeletePathRequest, DeletePathResponse, ReadVersionRequest, ReadVersionResponse,
    RenameDataRequest, RenameDataResponse, StorageRemoteError, WriteAllRequest, WriteAllResponse,
};
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

    fn unsupported(&self, method: &str) -> FS3Error {
        FS3Error::internal(format!(
            "remote storage method {method} is not implemented for endpoint {}",
            self.endpoint.address
        ))
    }

    fn transport_placeholder(&self, method: &str) -> FS3Error {
        FS3Error::internal(format!(
            "remote storage transport for {method} is not wired yet for endpoint {}",
            self.endpoint.address
        ))
    }

    fn route_url(&self, route: &str) -> String {
        format!(
            "{}/{}",
            self.endpoint.address.trim_end_matches('/'),
            route.trim_start_matches('/')
        )
    }

    async fn post_json<Req, Resp>(&self, route: &str, body: &Req) -> Result<Resp, FS3Error>
    where
        Req: serde::Serialize + ?Sized,
        Resp: serde::de::DeserializeOwned,
    {
        let response = self
            .http
            .post(self.route_url(route))
            .json(body)
            .send()
            .await
            .map_err(|err| FS3Error::internal(format!("remote storage request failed: {err}")))?;

        if response.status().is_success() {
            return response
                .json::<Resp>()
                .await
                .map_err(|err| FS3Error::internal(format!("remote storage response decode failed: {err}")));
        }

        let status = response.status();
        let remote = response.json::<StorageRemoteError>().await.ok();
        let message = remote
            .map(|err| err.message)
            .unwrap_or_else(|| format!("remote storage request failed with status {status}"));
        Err(FS3Error::new(status, message))
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

    async fn delete_vol(&self, _ctx: &Context, _volume: &str, _force: bool) -> Result<(), FS3Error> {
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
        let resp: ReadVersionResponse = self
            .post_json(
                "/__fs3/storage/read-version",
                &ReadVersionRequest {
                    volume: volume.to_string(),
                    path: path.to_string(),
                    version_id: version_id.to_string(),
                },
            )
            .await?;
        Ok(resp.file_info)
    }

    async fn write_all(
        &self,
        _ctx: &Context,
        volume: &str,
        path: &str,
        data: &[u8],
        opts: WriteAllOptions,
    ) -> Result<(), FS3Error> {
        let _: WriteAllResponse = self
            .post_json(
                "/__fs3/storage/write-all",
                &WriteAllRequest {
                    volume: volume.to_string(),
                    path: path.to_string(),
                    data: data.to_vec(),
                    opts,
                },
            )
            .await?;
        Ok(())
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
        src_volume: &str,
        src_path: &str,
        fi: FileInfo,
        dst_volume: &str,
        dst_path: &str,
        opts: RenameDataOptions,
    ) -> Result<RenameDataResult, FS3Error> {
        let resp: RenameDataResponse = self
            .post_json(
                "/__fs3/storage/rename-data",
                &RenameDataRequest {
                    src_volume: src_volume.to_string(),
                    src_path: src_path.to_string(),
                    file_info: fi,
                    dst_volume: dst_volume.to_string(),
                    dst_path: dst_path.to_string(),
                    opts,
                },
            )
            .await?;
        Ok(resp.result)
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

    async fn append_file(&self, _ctx: &Context, _volume: &str, _path: &str, _buf: &[u8]) -> Result<(), FS3Error> {
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
        let _: DeletePathResponse = self
            .post_json(
                "/__fs3/storage/delete-path",
                &DeletePathRequest {
                    volume: volume.to_string(),
                    path: path.to_string(),
                    opts,
                },
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl StorageBucketConfig<FS3Error> for RemoteStorageClient {
    async fn read_bucket_policy(&self, _ctx: &Context, _bucket: &str) -> Result<Option<String>, FS3Error> {
        Err(self.unsupported("read_bucket_policy"))
    }

    async fn write_bucket_policy(&self, _ctx: &Context, _bucket: &str, _policy: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("write_bucket_policy"))
    }

    async fn delete_bucket_policy(&self, _ctx: &Context, _bucket: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("delete_bucket_policy"))
    }

    async fn read_bucket_tags(&self, _ctx: &Context, _bucket: &str) -> Result<Option<String>, FS3Error> {
        Err(self.unsupported("read_bucket_tags"))
    }

    async fn write_bucket_tags(&self, _ctx: &Context, _bucket: &str, _tags: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("write_bucket_tags"))
    }

    async fn delete_bucket_tags(&self, _ctx: &Context, _bucket: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("delete_bucket_tags"))
    }

    async fn read_bucket_versioning(&self, _ctx: &Context, _bucket: &str) -> Result<Option<String>, FS3Error> {
        Err(self.unsupported("read_bucket_versioning"))
    }

    async fn write_bucket_versioning(&self, _ctx: &Context, _bucket: &str, _status: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("write_bucket_versioning"))
    }

    async fn read_bucket_cors(&self, _ctx: &Context, _bucket: &str) -> Result<Option<String>, FS3Error> {
        Err(self.unsupported("read_bucket_cors"))
    }

    async fn write_bucket_cors(&self, _ctx: &Context, _bucket: &str, _cors: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("write_bucket_cors"))
    }

    async fn delete_bucket_cors(&self, _ctx: &Context, _bucket: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("delete_bucket_cors"))
    }
}

#[async_trait]
impl StorageObjectConfig<FS3Error> for RemoteStorageClient {
    async fn read_object_tags(&self, _ctx: &Context, _bucket: &str, _key: &str) -> Result<Option<String>, FS3Error> {
        Err(self.unsupported("read_object_tags"))
    }

    async fn write_object_tags(&self, _ctx: &Context, _bucket: &str, _key: &str, _tags: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("write_object_tags"))
    }

    async fn delete_object_tags(&self, _ctx: &Context, _bucket: &str, _key: &str) -> Result<(), FS3Error> {
        Err(self.unsupported("delete_object_tags"))
    }
}
