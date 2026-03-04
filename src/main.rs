use clap::{Parser, Subcommand};
use s3_mount_gateway_rust::clis::{server::ServerArgs, run_server};

#[derive(Parser)]
#[command(name = "fs3")]
#[command(about = "S3-compatible object storage gateway")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Server(ServerArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Server(args) => run_server(args).await?,
    }
    
    Ok(())
}
