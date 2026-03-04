use minio::s3::types::S3Api;
use minio::s3::builders::ObjectContent;
use super::helpers::{create_minio_client, create_minio_server};
use http::Method;

#[tokio::test]
async fn test_presigned_get_url() {
    let (_addr, endpoint, _handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "presigned-test";

    client.create_bucket(bucket).send().await.unwrap();

    let key = "test-object";
    let data = b"test data with unique content 12345";
    client.put_object_content(bucket, key, ObjectContent::from(data.as_ref())).send().await.unwrap();

    let presigned_resp = client.get_presigned_object_url(bucket, key, Method::GET).send().await.unwrap();
    let presigned_url = presigned_resp.url;

    let resp = reqwest::get(&presigned_url).await.unwrap();
    assert!(resp.status().is_success(), "Presigned GET must return 200");
    assert_eq!(resp.bytes().await.unwrap().as_ref(), data, "Presigned GET must return exact data");
}

#[tokio::test]
async fn test_presigned_put_url() {
    let (_addr, endpoint, _handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "presigned-put-test";

    client.create_bucket(bucket).send().await.unwrap();

    let key = "test-upload";
    let data = b"uploaded data via presigned PUT";
    let presigned_resp = client.get_presigned_object_url(bucket, key, Method::PUT).send().await.unwrap();
    let presigned_url = presigned_resp.url;

    let http_client = reqwest::Client::new();
    let put_resp = http_client.put(&presigned_url).body(data.as_ref()).send().await.unwrap();
    assert!(put_resp.status().is_success(), "Presigned PUT must succeed");

    let obj = client.get_object(bucket, key).send().await.unwrap();
    let content = obj.content.to_segmented_bytes().await.unwrap().to_bytes();
    assert_eq!(content.as_ref(), data, "Presigned PUT must upload exact data");
}
