use minio::s3::builders::ObjectContent;
use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn error_scenarios_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();

    // 不存在的桶
    let err = client.bucket_exists("nonexistent").send().await.unwrap();
    assert!(!err.exists);

    // 不存在的对象
    let err = client.stat_object("nonexistent", "key").send().await;
    assert!(err.is_err());

    // 创建已存在的桶
    client.create_bucket("test").send().await.unwrap();
    let err = client.create_bucket("test").send().await;
    assert!(err.is_err());

    // 删除非空桶
    client.put_object_content("test", "file", ObjectContent::from(b"data".as_ref())).send().await.unwrap();
    let err = client.delete_bucket("test").send().await;
    assert!(err.is_err());

    // 清理
    client.delete_object("test", "file").send().await.unwrap();
    client.delete_bucket("test").send().await.unwrap();

    handle.abort();
}
