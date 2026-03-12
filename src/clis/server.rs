use crate::components::erasure_server_pools::ErasureServerPools;
use crate::components::fs3_axum_handler::S3AxumHandler;
use crate::components::fs3_engine::FS3Engine;
use crate::components::storage_policy::StoragePolicyEngine;
use crate::components::xl_storage::XlStorage;
use clap::Args;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Args)]
pub struct ServerArgs {
    #[arg(long, default_value = "127.0.0.1:9000")]
    pub address: String,

    pub paths: Vec<String>,
}

pub async fn run_server(args: ServerArgs) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mount_point = if !args.paths.is_empty() {
        args.paths[0].clone()
    } else {
        std::env::var("FS3_MOUNT_POINT").unwrap_or_else(|_| ".debug".to_string())
    };

    let storage = Arc::new(XlStorage::new(mount_point.into()));
    let object_layer = Arc::new(ErasureServerPools::new(storage.clone()));
    let engine = FS3Engine::new(object_layer, storage.clone());
    let policy = StoragePolicyEngine::new(storage.clone());
    let handler = S3AxumHandler::new(engine, policy);

    let listener = TcpListener::bind(&args.address).await?;
    println!("API: http://{}", args.address);

    if !args.paths.is_empty() {
        println!("Paths: {:?}", args.paths);
    }

    let app = crate::axum_router(handler);
    axum::serve(listener, app).await?;

    Ok(())
}
