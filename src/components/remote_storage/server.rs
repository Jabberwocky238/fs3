use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;

use crate::types::FS3Error;
use crate::types::s3::object_layer_types::Context;
use crate::types::storage_endpoint::StorageEndpoint;
use crate::types::storage_remote::{
    DeletePathRequest, DeletePathResponse, ReadVersionRequest, ReadVersionResponse,
    RenameDataRequest, RenameDataResponse, StorageRemoteError, WriteAllRequest, WriteAllResponse,
};
use crate::types::traits::storage_api::StorageAPI;

pub struct RemoteStorageServer {
    endpoint: StorageEndpoint,
    storage: Arc<dyn StorageAPI<FS3Error>>,
}

impl RemoteStorageServer {
    pub fn new(endpoint: StorageEndpoint, storage: Arc<dyn StorageAPI<FS3Error>>) -> Self {
        Self { endpoint, storage }
    }

    pub fn endpoint(&self) -> &StorageEndpoint {
        &self.endpoint
    }

    pub fn storage(&self) -> &Arc<dyn StorageAPI<FS3Error>> {
        &self.storage
    }

    pub fn router(storage: Arc<dyn StorageAPI<FS3Error>>) -> Router {
        let state = Arc::new(storage);
        Router::new()
            .route("/__fs3/storage/read-version", post(read_version))
            .route("/__fs3/storage/write-all", post(write_all))
            .route("/__fs3/storage/rename-data", post(rename_data))
            .route("/__fs3/storage/delete-path", post(delete_path))
            .with_state(state)
    }
}

fn remote_error(err: FS3Error) -> (StatusCode, Json<StorageRemoteError>) {
    (
        err.status(),
        Json(StorageRemoteError {
            code: err.status().as_str().to_string(),
            message: err.message().to_string(),
        }),
    )
}

fn remote_context() -> Context {
    Context {
        request_id: "remote-storage".to_string(),
    }
}

async fn read_version(
    State(storage): State<Arc<Arc<dyn StorageAPI<FS3Error>>>>,
    Json(req): Json<ReadVersionRequest>,
) -> Result<Json<ReadVersionResponse>, (StatusCode, Json<StorageRemoteError>)> {
    let file_info = storage
        .read_version(&remote_context(), &req.volume, &req.path, &req.version_id)
        .await
        .map_err(remote_error)?;
    Ok(Json(ReadVersionResponse { file_info }))
}

async fn write_all(
    State(storage): State<Arc<Arc<dyn StorageAPI<FS3Error>>>>,
    Json(req): Json<WriteAllRequest>,
) -> Result<Json<WriteAllResponse>, (StatusCode, Json<StorageRemoteError>)> {
    storage
        .write_all(&remote_context(), &req.volume, &req.path, &req.data, req.opts)
        .await
        .map_err(remote_error)?;
    Ok(Json(WriteAllResponse))
}

async fn rename_data(
    State(storage): State<Arc<Arc<dyn StorageAPI<FS3Error>>>>,
    Json(req): Json<RenameDataRequest>,
) -> Result<Json<RenameDataResponse>, (StatusCode, Json<StorageRemoteError>)> {
    let result = storage
        .rename_data(
            &remote_context(),
            &req.src_volume,
            &req.src_path,
            req.file_info,
            &req.dst_volume,
            &req.dst_path,
            req.opts,
        )
        .await
        .map_err(remote_error)?;
    Ok(Json(RenameDataResponse { result }))
}

async fn delete_path(
    State(storage): State<Arc<Arc<dyn StorageAPI<FS3Error>>>>,
    Json(req): Json<DeletePathRequest>,
) -> Result<Json<DeletePathResponse>, (StatusCode, Json<StorageRemoteError>)> {
    storage
        .delete_path(&remote_context(), &req.volume, &req.path, req.opts)
        .await
        .map_err(remote_error)?;
    Ok(Json(DeletePathResponse))
}
