use minio::s3::builders::{ObjectContent, VersioningStatus};
use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn versioning_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "version-bucket";
    let key = "versioned.txt";

    client.create_bucket(bucket).send().await.unwrap();

    // 启用版本控制
    client.put_bucket_versioning(bucket).versioning_status(VersioningStatus::Enabled).send().await.unwrap();

    // 上传多个版本
    client.put_object_content(bucket, key, ObjectContent::from(b"v1".as_ref())).send().await.unwrap();
    client.put_object_content(bucket, key, ObjectContent::from(b"v2".as_ref())).send().await.unwrap();
    client.put_object_content(bucket, key, ObjectContent::from(b"v3".as_ref())).send().await.unwrap();

    // 获取最新版本
    let resp = client.get_object(bucket, key).send().await.unwrap();
    let content = resp.content.to_segmented_bytes().await.unwrap().to_bytes();
    assert_eq!(content.as_ref(), b"v3");

    // 删除对象
    client.delete_object(bucket, key).send().await.unwrap();

    // 清理
    client.delete_bucket(bucket).send().await.unwrap();

    handle.abort();
}
