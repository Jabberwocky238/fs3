use minio::s3::types::{S3Api, ToStream};
use minio::s3::builders::ObjectContent;
use futures::StreamExt;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn list_operations_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "list-test";

    client.create_bucket(bucket).send().await.unwrap();

    for i in 1..=5 {
        client.put_object_content(bucket, &format!("f{}.txt", i), ObjectContent::from(b"x".as_ref())).send().await.unwrap();
    }

    // ListObjectsV2 stream
    let mut stream = client.list_objects(bucket).recursive(true).to_stream().await;
    let mut count = 0;
    while let Some(result) = stream.next().await {
        count += result.unwrap().contents.len();
    }
    assert_eq!(count, 5);

    for i in 1..=5 {
        client.delete_object(bucket, &format!("f{}.txt", i)).send().await.unwrap();
    }
    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
