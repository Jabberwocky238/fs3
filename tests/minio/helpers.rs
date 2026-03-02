use std::net::SocketAddr;

use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::error::Error as MinioError;
use minio::s3::http::BaseUrl;
use s3_mount_gateway_rust::axum_router;
use s3_mount_gateway_rust::components::s3_engine::S3EngineImpl;
use s3_mount_gateway_rust::types::errors::S3EngineError;
use s3_mount_gateway_rust::components::s3_metadata_storage::memory::MemoryMetadataStorage;
use s3_mount_gateway_rust::components::s3_mount::memory::MemoryMount;
use s3_mount_gateway_rust::components::s3_axum_handler::S3AxumHandler;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub const MINIO_ACCESS_KEY: &str = "minioadmin";
pub const MINIO_SECRET_KEY: &str = "minioadmin";

pub async fn create_minio_server() -> std::io::Result<(SocketAddr, String, JoinHandle<()>)> {
    let engine = S3EngineImpl::new(MemoryMetadataStorage::new(), MemoryMount::new());
    let handler = S3AxumHandler::new(engine);
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
