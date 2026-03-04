use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::presigning::PresigningConfig;
use super::helpers::{create_aws_client, create_test_server};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread")]
async fn test_presigned_get_url() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "presigned-test";

    client.create_bucket().bucket(bucket).send().await.unwrap();

    let key = "test-object";
    let data = b"test data with unique content 12345";
    client.put_object().bucket(bucket).key(key).body(ByteStream::from_static(data)).send().await.unwrap();

    let presigned = client.get_object()
        .bucket(bucket)
        .key(key)
        .presigned(PresigningConfig::expires_in(Duration::from_secs(3600)).unwrap())
        .await.unwrap();

    let resp = reqwest::get(presigned.uri()).await.unwrap();
    assert!(resp.status().is_success());
    assert_eq!(resp.bytes().await.unwrap().as_ref(), data);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_presigned_put_url() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "presigned-put-test";

    client.create_bucket().bucket(bucket).send().await.unwrap();

    let key = "test-upload";
    let data = b"uploaded data via presigned PUT";
    let presigned = client.put_object()
        .bucket(bucket)
        .key(key)
        .presigned(PresigningConfig::expires_in(Duration::from_secs(3600)).unwrap())
        .await.unwrap();

    let http_client = reqwest::Client::new();
    let put_resp = http_client.put(presigned.uri()).body(data.as_ref()).send().await.unwrap();
    assert!(put_resp.status().is_success());

    let obj = client.get_object().bucket(bucket).key(key).send().await.unwrap();
    let content = obj.body.collect().await.unwrap().to_vec();
    assert_eq!(content.as_slice(), data);
}
