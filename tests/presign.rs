#![cfg(all(feature = "policy", feature = "multi-user"))]
mod helpers;

use minio::s3::types::S3Api;
use reqwest::StatusCode;

use helpers::*;
use s3_mount_gateway_rust::config::{Config, MountOptions, StorageKind, StorageOptions};

#[tokio::test(flavor = "multi_thread")]
async fn presign_upload_download_and_multipart() {
    let inner_port = free_port();
    let outer_port = free_port();
    let conf = Config {
        listen_inner: format!("127.0.0.1:{inner_port}"),
        listen_outer: format!("127.0.0.1:{outer_port}"),
        multi_bucket_enabled: false,
        mount: MountOptions::memory(),
        storage: StorageOptions {
            kind: StorageKind::Memory,
            ..StorageOptions::default()
        },
        ..Default::default()
    };

    let (base, handle) = helpers::start_test_server("presign", Some(conf)).await;
    let minio = helpers::minio_client(&base, "fs3_admin-ak", "fs3_admin-sk");
    let http = reqwest::Client::new();

    let presign_up = http
        .post(format!("{}/api/presign/upload", base))
        .json(&serde_json::json!({
            "bucket":"docs",
            "key":"team-a/presign-upload.txt",
            "expires_seconds": 300
        }))
        .send()
        .await
        .expect("presign upload failed");
    assert_eq!(presign_up.status(), StatusCode::OK);
    let up_json: serde_json::Value = presign_up
        .json()
        .await
        .expect("parse presign upload json failed");
    let up_url = up_json["url"].as_str().expect("missing upload url");

    let put = http
        .put(up_url)
        .body("hello-presign-upload")
        .send()
        .await
        .expect("put by presigned url failed");
    assert_eq!(put.status(), StatusCode::OK);

    let uploaded = minio
        .get_object("docs", "team-a/presign-upload.txt")
        .send()
        .await
        .expect("minio get uploaded object failed");
    let uploaded_bytes = uploaded
        .content
        .to_segmented_bytes()
        .await
        .expect("read uploaded object bytes failed")
        .to_bytes();
    assert_eq!(uploaded_bytes.as_ref(), b"hello-presign-upload");

    minio
        .put_object_content(
            "docs",
            "team-a/presign-download.txt",
            "hello-presign-download".to_string(),
        )
        .send()
        .await
        .expect("minio put download source failed");

    let presign_down = http
        .post(format!("{}/api/presign/download", base))
        .json(&serde_json::json!({
            "bucket":"docs",
            "key":"team-a/presign-download.txt",
            "expires_seconds": 300
        }))
        .send()
        .await
        .expect("presign download failed");
    assert_eq!(presign_down.status(), StatusCode::OK);
    let down_json: serde_json::Value = presign_down
        .json()
        .await
        .expect("parse presign download json failed");
    let down_url = down_json["url"].as_str().expect("missing download url");

    let got = http
        .get(down_url)
        .send()
        .await
        .expect("download by presigned url failed");
    assert_eq!(got.status(), StatusCode::OK);
    assert_eq!(
        got.text().await.expect("read downloaded text failed"),
        "hello-presign-download"
    );

    let init = http
        .post(format!("{}/api/multipart/init", base))
        .json(&serde_json::json!({"bucket":"docs","key":"team-a/multi.bin"}))
        .send()
        .await
        .expect("multipart init failed");
    assert_eq!(init.status(), StatusCode::OK);
    let init_json: serde_json::Value = init.json().await.expect("parse init json failed");
    let upload_id = init_json["upload_id"].as_str().expect("missing upload_id");

    for (pn, body) in [(1, "abc"), (2, "def")] {
        let part = http
            .post(format!("{}/api/multipart/presign-part", base))
            .json(&serde_json::json!({
                "upload_id": upload_id,
                "part_number": pn,
                "expires_seconds": 300
            }))
            .send()
            .await
            .expect("multipart presign part failed");
        assert_eq!(part.status(), StatusCode::OK);
        let pjson: serde_json::Value = part.json().await.expect("parse part json failed");
        let purl = pjson["url"].as_str().expect("missing part url");

        let uploaded = http
            .put(purl)
            .body(body)
            .send()
            .await
            .expect("upload part failed");
        assert_eq!(uploaded.status(), StatusCode::OK);
    }

    let complete = http
        .post(format!("{}/api/multipart/complete", base))
        .json(&serde_json::json!({"upload_id": upload_id, "parts": [1,2]}))
        .send()
        .await
        .expect("multipart complete failed");
    assert_eq!(complete.status(), StatusCode::OK);

    let merged = minio
        .get_object("docs", "team-a/multi.bin")
        .send()
        .await
        .expect("minio get multipart object failed");
    let merged_bytes = merged
        .content
        .to_segmented_bytes()
        .await
        .expect("read multipart object bytes failed")
        .to_bytes();
    assert_eq!(merged_bytes.as_ref(), b"abcdef");

    handle.abort();
}
