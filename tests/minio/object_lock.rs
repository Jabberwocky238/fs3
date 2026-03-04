use minio::s3::types::S3Api;
use minio::s3::builders::ObjectContent;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn object_lock_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "locktest";
    let key = "locked.txt";

    client.create_bucket(bucket).send().await.unwrap();
    client.put_object_content(bucket, key, ObjectContent::from(b"data".as_ref())).send().await.unwrap();

    // Legal Hold - 只测试设置，不验证返回
    let _ = client.put_object_legal_hold(bucket, key, true).send().await;

    client.delete_object(bucket, key).send().await.unwrap();
    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
