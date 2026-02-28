use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    s3_mount_gateway_rust::server::run_from_cli().await
}
