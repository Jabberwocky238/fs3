use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::http::StatusCode;
use chrono::Utc;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::types::S3Api;
use minio::s3::{Client, ClientBuilder};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use s3_mount_gateway_rust::axum_router;
use s3_mount_gateway_rust::types::s3::request::*;
use s3_mount_gateway_rust::types::s3::response::*;
use s3_mount_gateway_rust::types::traits::s3_handler::{BucketS3Handler, ObjectS3Handler, RejectedS3Handler, RootS3Handler};

async fn start_server() -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind real port failed");
    let addr = listener.local_addr().expect("local addr failed");
    let base = format!("http://{}", addr);

    let app = axum_router::<RealAxumHandler, HandlerErr>(RealAxumHandler::default());
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let client = reqwest::Client::new();
    for _ in 0..100 {
        if client.get(format!("{base}/")).send().await.is_ok() {
            return (base, handle);
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("server not ready")
}

fn minio_client(base: &str, access_key: &str, secret_key: &str) -> Client {
    let base_url = base.parse::<BaseUrl>().expect("invalid base url");
    let provider = StaticProvider::new(access_key, secret_key, None);
    ClientBuilder::new(base_url)
        .provider(Some(Box::new(provider)))
        .build()
        .expect("build minio client failed")
}

#[tokio::test(flavor = "multi_thread")]
async fn real_axum_handler_real_start_real_port() {
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();
    let minio = minio_client(&base, "ak-test", "sk-test");
    let payload_text = "hello-real-axum-from-minio";

    minio
        .create_bucket("docs")
        .send()
        .await
        .expect("minio create_bucket failed");
    minio
        .put_object_content("docs", "hello.txt", payload_text.to_string())
        .send()
        .await
        .expect("minio put_object_content failed");

    let get_object = client
        .get(format!("{base}/docs/hello.txt"))
        .send()
        .await
        .expect("get object failed");
    assert_eq!(get_object.status(), StatusCode::OK);
    let payload: serde_json::Value = get_object
        .json()
        .await
        .expect("parse get json failed");
    assert_eq!(payload["api"], "GetObject");
    assert_eq!(
        payload["response"]["body"],
        serde_json::json!(payload_text.as_bytes())
    );

    let list_buckets = client
        .get(format!("{base}/"))
        .send()
        .await
        .expect("list buckets failed");
    assert_eq!(list_buckets.status(), StatusCode::OK);
    let root: serde_json::Value = list_buckets
        .json()
        .await
        .expect("parse list buckets json failed");
    assert_eq!(root["api"], "ListBuckets");
    assert_eq!(root["response"]["buckets"][0]["name"], "docs");

    handle.abort();
}
