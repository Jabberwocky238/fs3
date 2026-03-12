use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use aws_sdk_s3::Client as AwsClient;
use aws_sdk_s3::config::{Credentials, Region};
use s3_mount_gateway_rust::axum_router;
use s3_mount_gateway_rust::components::erasure_server_pools::ErasureServerPools;
use s3_mount_gateway_rust::components::fs3_axum_handler::S3AxumHandler;
use s3_mount_gateway_rust::components::fs3_engine::FS3Engine;
use s3_mount_gateway_rust::components::storage_policy::StoragePolicyEngine;
use s3_mount_gateway_rust::components::xl_storage::XlStorage;
use s3_mount_gateway_rust::types::errors::S3EngineError;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub const AWS_ACCESS_KEY: &str = "minioadmin";
pub const AWS_SECRET_KEY: &str = "minioadmin";

pub async fn create_test_server() -> std::io::Result<(SocketAddr, String, JoinHandle<()>)> {
    let storage = Arc::new(XlStorage::from_env());
    let object_layer = Arc::new(ErasureServerPools::new(storage.clone()));
    let engine = FS3Engine::new(object_layer, storage.clone());
    let policy = StoragePolicyEngine::new(storage);
    let handler = S3AxumHandler::new(engine, policy);
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let addr = listener.local_addr()?;
    let endpoint = format!("http://{addr}");
    let app = axum_router::<_, S3EngineError>(handler).layer(axum::middleware::from_fn(
        |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| async move {
            println!("Request: {} {}", req.method(), req.uri());
            next.run(req).await
        },
    ));

    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    Ok((addr, endpoint, task))
}

pub fn create_aws_client(endpoint: &str) -> AwsClient {
    let creds = Credentials::new(AWS_ACCESS_KEY, AWS_SECRET_KEY, None, None, "static");
    let config = aws_sdk_s3::Config::builder()
        .endpoint_url(endpoint)
        .region(Region::new("us-east-1"))
        .credentials_provider(creds)
        .force_path_style(true)
        .behavior_version_latest()
        .build();
    AwsClient::from_conf(config)
}

pub fn random_bucket_name() -> String {
    format!(
        "test-{}",
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
    )
}
