use super::helpers::{create_aws_client, create_test_server};
use aws_sdk_s3::primitives::ByteStream;

#[tokio::test(flavor = "multi_thread")]
async fn test_get_object_if_match() {
    let (_addr, endpoint, _task) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "test-conditional";

    client.create_bucket().bucket(bucket).send().await.unwrap();
    client
        .put_object()
        .bucket(bucket)
        .key("test.txt")
        .body(ByteStream::from_static(b"hello"))
        .send()
        .await
        .unwrap();

    let head = client
        .head_object()
        .bucket(bucket)
        .key("test.txt")
        .send()
        .await
        .unwrap();
    let etag = head.e_tag().unwrap();

    let http_client = reqwest::Client::new();
    let url = format!("{}/{}/{}", endpoint, bucket, "test.txt");

    let resp = http_client
        .get(&url)
        .header("if-match", etag)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let resp = http_client
        .get(&url)
        .header("if-match", "wrong-etag")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 412);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_object_if_none_match() {
    let (_addr, endpoint, _task) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "test-conditional2";

    client.create_bucket().bucket(bucket).send().await.unwrap();
    client
        .put_object()
        .bucket(bucket)
        .key("test.txt")
        .body(ByteStream::from_static(b"hello"))
        .send()
        .await
        .unwrap();

    let head = client
        .head_object()
        .bucket(bucket)
        .key("test.txt")
        .send()
        .await
        .unwrap();
    let etag = head.e_tag().unwrap();

    let http_client = reqwest::Client::new();
    let url = format!("{}/{}/{}", endpoint, bucket, "test.txt");

    let resp = http_client
        .get(&url)
        .header("if-none-match", etag)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 304);

    let resp = http_client
        .get(&url)
        .header("if-none-match", "wrong-etag")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}
