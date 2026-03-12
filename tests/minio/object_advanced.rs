use minio::s3::builders::ObjectContent;
use minio::s3::types::S3Api;
use std::collections::HashMap;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn object_advanced_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "advanced-bucket";
    let key = "test.txt";
    let data = b"0123456789abcdefghij";

    client.create_bucket(bucket).send().await.unwrap();

    // 上传对象
    client
        .put_object_content(bucket, key, ObjectContent::from(data.as_ref()))
        .send()
        .await
        .unwrap();

    // 对象标签
    let mut tags = HashMap::new();
    tags.insert("env".to_string(), "test".to_string());
    client
        .put_object_tagging(bucket, key)
        .tags(tags)
        .send()
        .await
        .unwrap();
    let got_tags = client.get_object_tagging(bucket, key).send().await.unwrap();
    assert!(!got_tags.tags.is_empty());

    // 删除标签
    client
        .delete_object_tagging(bucket, key)
        .send()
        .await
        .unwrap();

    // 清理
    client.delete_object(bucket, key).send().await.unwrap();
    client.delete_bucket(bucket).send().await.unwrap();

    handle.abort();
}
