use anyhow::Result;
use clap::Parser;
use s3_mount_gateway_rust::{config::Config, server::{S3Server, load_config}};
use tracing::info;

#[derive(Parser)]
#[command(name = "fs3", about = "S3-compatible mount gateway")]
struct Cli {
    /// Config file path (JSON)
    #[arg(short, long)]
    config: Option<String>,

    /// Inner listen address
    #[arg(short, long)]
    listen: Option<String>,
}

pub async fn run_from_cli() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    let mut cfg = match cli.config {
        Some(ref path) => {
            info!(path = %path, "loading config file");
            load_config(path)?
        }
        None => {
            info!("no config file specified, using defaults");
            Config::default()
        }
    };

    if let Some(v) = cli.listen { cfg.listen_inner = v; }

    info!(listen_inner = %cfg.listen_inner, listen_outer = %cfg.listen_outer,
          mount_mode = ?cfg.mount.mode, storage_kind = ?cfg.storage.kind,
          "resolved config");

    S3Server::from_config(cfg).await?.serve().await
}

#[tokio::main]
async fn main() -> Result<()> {
    run_from_cli().await
}
