use minio::s3::builders::ObjectContent;
use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn test_get_object_if_match() {
    let (_addr, endpoint, _task) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "test-conditional";

    client.create_bucket(bucket).send().await.unwrap();
    client.put_object_content(bucket, "test.txt", ObjectContent::from(b"hello".as_ref())).send().await.unwrap();

    let obj = client.stat_object(bucket, "test.txt").send().await.unwrap();
    let etag = obj.etag;

    let http_client = reqwest::Client::new();
    let url = format!("{}/{}/{}", endpoint, bucket, "test.txt");

    let resp = http_client.get(&url).header("if-match", &etag).send().await.unwrap();
    let status = resp.status();
    if status != 200 {
        let body = resp.text().await.unwrap();
        panic!("Expected 200, got {}: {}", status, body);
    }

    let resp = http_client.get(&url).header("if-match", "wrong-etag").send().await.unwrap();
    let status = resp.status();
    if status != 412 {
        let body = resp.text().await.unwrap();
        panic!("Expected 412, got {}: {}", status, body);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_object_if_none_match() {
    let (_addr, endpoint, _task) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "test-conditional2";

    client.create_bucket(bucket).send().await.unwrap();
    client.put_object_content(bucket, "test.txt", ObjectContent::from(b"hello".as_ref())).send().await.unwrap();

    let obj = client.stat_object(bucket, "test.txt").send().await.unwrap();
    let etag = obj.etag;

    let http_client = reqwest::Client::new();
    let url = format!("{}/{}/{}", endpoint, bucket, "test.txt");

    let resp = http_client.get(&url).header("if-none-match", &etag).send().await.unwrap();
    let status = resp.status();
    if status != 304 {
        let body = resp.text().await.unwrap();
        panic!("Expected 304, got {}: {}", status, body);
    }

    let resp = http_client.get(&url).header("if-none-match", "wrong-etag").send().await.unwrap();
    assert_eq!(resp.status(), 200);
}
