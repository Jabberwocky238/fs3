use std::net::SocketAddr;

use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::error::Error as MinioError;
use minio::s3::http::BaseUrl;
use s3_mount_gateway_rust::axum_router;
use s3_mount_gateway_rust::components::fs3_engine::FS3Engine;
use s3_mount_gateway_rust::types::errors::S3EngineError;
use s3_mount_gateway_rust::components::s3_metadata_storage::sqlite::{SqliteMetadataStorage, SQLITE_MEMORY};
use s3_mount_gateway_rust::components::s3_mount::memory::MemoryMount;
use s3_mount_gateway_rust::components::fs3_axum_handler::S3AxumHandler;
use s3_mount_gateway_rust::components::fs3_policyengine::DefaultPolicyEngine;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub const MINIO_ACCESS_KEY: &str = "minioadmin";
pub const MINIO_SECRET_KEY: &str = "minioadmin";

pub async fn create_minio_server() -> std::io::Result<(SocketAddr, String, JoinHandle<()>)> {
    let metadata_storage = SqliteMetadataStorage::new(SQLITE_MEMORY).await.unwrap();
    let engine = FS3Engine::new(metadata_storage, MemoryMount::new());
    let handler = S3AxumHandler::new(engine, DefaultPolicyEngine);
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let addr = listener.local_addr()?;
    let endpoint = format!("http://{addr}");
    let app = axum_router::<_, S3EngineError>(handler);

    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    Ok((addr, endpoint, task))
}

pub fn create_minio_client(endpoint: &str) -> Result<Client, MinioError> {
    let base_url: BaseUrl = endpoint.parse()?;
    let provider = StaticProvider::new(MINIO_ACCESS_KEY, MINIO_SECRET_KEY, None);
    Client::new(base_url, Some(Box::new(provider)), None, Some(true))
}
