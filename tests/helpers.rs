use std::fs;
use std::net::TcpListener;
use std::time::Duration;

use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::{Client, ClientBuilder};
use reqwest::StatusCode;

use s3_mount_gateway_rust::config::{Config, MountOptions, StorageKind, StorageOptions};
use s3_mount_gateway_rust::server::S3Server;

pub async fn start_test_server(
    tag: &str,
    conf: Option<Config>,
) -> (String, tokio::task::JoinHandle<()>) {
    let mut root = std::env::temp_dir();
    root.push(format!("s3gw-{tag}-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).expect("create root dir failed");

    let cfg = match conf {
        Some(c) => c,
        None => {
            let inner_port = free_port();
            let outer_port = free_port();
            Config {
                listen_inner: format!("127.0.0.1:{inner_port}"),
                listen_outer: format!("127.0.0.1:{outer_port}"),
                multi_bucket_enabled: false,
                mount: MountOptions::memory(),
                storage: StorageOptions {
                    kind: StorageKind::Memory,
                    ..StorageOptions::default()
                },
                ..Default::default()
            }
        }
    };

    let server = S3Server::from_config(cfg)
        .await
        .expect("init S3Server failed");

    let base = format!("http://{}", server.listen_outer);
    let handle = tokio::spawn(async move {
        let _ = server.serve().await;
    });

    wait_ready(&base).await;
    (base, handle)
}

pub fn minio_client(base: &str, access_key: &str, secret_key: &str) -> Client {
    let base_url = base.parse::<BaseUrl>().expect("invalid base url");
    let provider = StaticProvider::new(access_key, secret_key, None);
    ClientBuilder::new(base_url)
        .provider(Some(Box::new(provider)))
        .build()
        .expect("build minio client failed")
}

pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port failed")
        .local_addr()
        .expect("read local addr failed")
        .port()
}

async fn wait_ready(base: &str) {
    let client = reqwest::Client::new();
    for _ in 0..100 {
        if let Ok(resp) = client.get(format!("{base}/healthz")).send().await
            && resp.status() == StatusCode::OK
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("server not ready");
}
