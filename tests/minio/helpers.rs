use std::net::SocketAddr;

use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::error::Error as MinioError;
use minio::s3::http::BaseUrl;
use s3_mount_gateway_rust::axum_router;
use s3_mount_gateway_rust::components::s3_engine::memory::MemoryS3Engine;
use s3_mount_gateway_rust::types::traits::s3_handler::Handler;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub const MINIO_ACCESS_KEY: &str = "minioadmin";
pub const MINIO_SECRET_KEY: &str = "minioadmin";

pub async fn create_minio_server() -> std::io::Result<(SocketAddr, String, JoinHandle<()>)> {
    let engine = MemoryS3Engine::new();
    let handler = Handler::new(engine);
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let addr = listener.local_addr()?;
    let endpoint = format!("http://{addr}");
    let app = axum_router(handler);

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
