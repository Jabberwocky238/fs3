#![cfg(all(feature = "policy", feature = "multi-user"))]
mod helpers;

use minio::s3::builders::ObjectToDelete;
use minio::s3::types::S3Api;
use s3_mount_gateway_rust::config::{Config, MountOptions, StorageKind, StorageOptions};

use helpers::*;

#[tokio::test(flavor = "multi_thread")]
async fn single_bucket_mode() {
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
    let (base, handle) = helpers::start_test_server("single", Some(conf)).await;
    let client = helpers::minio_client(&base, "fs3_admin-ak", "fs3_admin-sk");

    // Creating a bucket should fail
    let create_err = client
        .create_bucket("newbucket")
        .send()
        .await
        .err();
    assert!(create_err.is_some(), "create_bucket should fail in single-bucket mode");

    // Deleting a bucket should fail
    let delete_err = client
        .delete_bucket("default")
        .send()
        .await
        .err();
    assert!(delete_err.is_some(), "delete_bucket should fail in single-bucket mode");

    // PUT object on default bucket should work
    let key = "test/hello.txt";
    let payload = "hello single bucket".to_string();
    client
        .put_object_content("default", key, payload.clone())
        .send()
        .await
        .expect("put_object_content on default bucket failed");

    // GET object
    let got = client
        .get_object("default", key)
        .send()
        .await
        .expect("get_object failed");
    let bytes = got
        .content
        .to_segmented_bytes()
        .await
        .expect("read bytes failed")
        .to_bytes();
    assert_eq!(String::from_utf8(bytes.to_vec()).unwrap(), payload);

    // HEAD object
    client
        .stat_object("default", key)
        .send()
        .await
        .expect("stat_object failed");

    // DELETE object
    client
        .delete_object("default", ObjectToDelete::from(key))
        .send()
        .await
        .expect("delete_object failed");

    handle.abort();
}
