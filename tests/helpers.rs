use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::{Client, ClientBuilder};
use reqwest::StatusCode;

use s3_mount_gateway_rust::config::{Config, MountOptions, StorageOptions, StorageKind};
use s3_mount_gateway_rust::server::S3Server;

pub struct TestServer {
    pub base: String,
    pub _root: PathBuf,
    _handle: tokio::task::JoinHandle<()>,
}

pub async fn start_test_server(tag: &str) -> TestServer {
    let mut root = std::env::temp_dir();
    root.push(format!("s3gw-{tag}-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).expect("create root dir failed");

    let inner_port = free_port();

    let cfg = Config {
        listen_inner: format!(":{inner_port}"),
        listen_outer: format!("127.0.0.1:{inner_port}"),
        mount: MountOptions::default(),
        policy_groups: vec![],
        storage: StorageOptions {
            kind: StorageKind::Json,
            json_path: root.join("storage.json").to_string_lossy().to_string(),
            dsn: String::new(),
            configmap_name: String::new(),
        },
    };

    let server = S3Server::from_config(cfg)
        .await
        .expect("init S3Server failed");

    let base = format!("http://127.0.0.1:{inner_port}");
    let handle = tokio::spawn(async move {
        let _ = server.serve().await;
    });

    wait_ready(&base).await;

    TestServer {
        base,
        _root: root,
        _handle: handle,
    }
}

pub fn minio_client(base: &str, access_key: &str, secret_key: &str) -> Client {
    let base_url = base.parse::<BaseUrl>().expect("invalid base url");
    let provider = StaticProvider::new(access_key, secret_key, None);
    ClientBuilder::new(base_url)
        .provider(Some(Box::new(provider)))
        .build()
        .expect("build minio client failed")
}

fn free_port() -> u16 {
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
