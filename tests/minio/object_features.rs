use minio::s3::builders::ObjectContent;
use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn object_features_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "features";
    let key = "test.txt";
    let data = b"0123456789abcdefghij";

    client.create_bucket(bucket).send().await.unwrap();
    client
        .put_object_content(bucket, key, ObjectContent::from(data.as_ref()))
        .send()
        .await
        .unwrap();

    // 基本读取
    let resp = client.get_object(bucket, key).send().await.unwrap();
    let content = resp.content.to_segmented_bytes().await.unwrap().to_bytes();
    assert_eq!(content.len(), 20);

    client.delete_object(bucket, key).send().await.unwrap();
    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
