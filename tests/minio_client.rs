#![cfg(all(feature = "policy", feature = "multi-user"))]
mod helpers;

use minio::s3::builders::ObjectToDelete;
use minio::s3::types::S3Api;

use helpers::free_port;
use s3_mount_gateway_rust::config::{Config, MountOptions, StorageKind, StorageOptions};

#[tokio::test(flavor = "multi_thread")]
async fn minio_client_smoke() {
    let inner_port = free_port();
    let outer_port = free_port();
    let conf = Config {
        listen_inner: format!("127.0.0.1:{inner_port}"),
        listen_outer: format!("127.0.0.1:{outer_port}"),
        multi_bucket_enabled: true,
        mount: MountOptions::memory(),
        storage: StorageOptions {
            kind: StorageKind::Memory,
            ..StorageOptions::default()
        },
        ..Default::default()
    };

    let (base, handle) = helpers::start_test_server("minio", Some(conf)).await;
    let client = helpers::minio_client(&base, "fs3_admin-ak", "fs3_admin-sk");

    let bucket = "docs";
    let key = "team-a/minio-rust-sdk.txt";
    let payload = "hello from minio rust sdk".to_string();

    client
        .create_bucket(bucket)
        .send()
        .await
        .expect("create_bucket failed");

    client
        .put_object_content(bucket, key, payload.clone())
        .send()
        .await
        .expect("put_object_content failed");

    let got = client
        .get_object(bucket, key)
        .send()
        .await
        .expect("get_object failed");

    let got_bytes = got
        .content
        .to_segmented_bytes()
        .await
        .expect("read object bytes failed")
        .to_bytes();
    let got_text = String::from_utf8(got_bytes.to_vec()).expect("object content is not valid utf8");
    assert_eq!(got_text, payload);

    client
        .stat_object(bucket, key)
        .send()
        .await
        .expect("stat_object failed");

    client
        .delete_object(bucket, ObjectToDelete::from(key))
        .send()
        .await
        .expect("delete_object failed");

    handle.abort();
}
