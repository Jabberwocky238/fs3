use minio::s3::builders::ObjectContent;
use minio::s3::types::{S3Api, ToStream};
use futures::StreamExt;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn list_objects_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "list-bucket";

    client.create_bucket(bucket).send().await.unwrap();

    // 创建测试对象
    client.put_object_content(bucket, "a/1.txt", ObjectContent::from(b"test".as_ref())).send().await.unwrap();
    client.put_object_content(bucket, "a/2.txt", ObjectContent::from(b"test".as_ref())).send().await.unwrap();
    client.put_object_content(bucket, "b/1.txt", ObjectContent::from(b"test".as_ref())).send().await.unwrap();
    client.put_object_content(bucket, "c.txt", ObjectContent::from(b"test".as_ref())).send().await.unwrap();

    // ListObjects - 使用递归模式
    let mut stream = client.list_objects(bucket).recursive(true).to_stream().await;
    let mut count = 0;
    while let Some(result) = stream.next().await {
        let response = result.unwrap();
        count += response.contents.len();
    }
    assert_eq!(count, 4);

    // 清理
    client.delete_object(bucket, "a/1.txt").send().await.unwrap();
    client.delete_object(bucket, "a/2.txt").send().await.unwrap();
    client.delete_object(bucket, "b/1.txt").send().await.unwrap();
    client.delete_object(bucket, "c.txt").send().await.unwrap();
    client.delete_bucket(bucket).send().await.unwrap();

    handle.abort();
}
