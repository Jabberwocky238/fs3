use minio::s3::types::S3Api;
use minio::s3::builders::{ObjectContent, VersioningStatus};

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn batch_and_version_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "bv";

    client.create_bucket(bucket).send().await.unwrap();

    // 批量删除
    for i in 1..=3 {
        client.put_object_content(bucket, &format!("f{}.txt", i), ObjectContent::from(b"x".as_ref())).send().await.unwrap();
    }
    client.remove_objects(bucket, vec!["f1.txt", "f2.txt", "f3.txt"]).send().await.unwrap();

    // 版本控制
    client.put_bucket_versioning(bucket).versioning_status(VersioningStatus::Enabled).send().await.unwrap();

    let key = "v.txt";
    client.put_object_content(bucket, key, ObjectContent::from(b"v1".as_ref())).send().await.unwrap();
    client.put_object_content(bucket, key, ObjectContent::from(b"v2".as_ref())).send().await.unwrap();

    client.delete_object(bucket, key).send().await.unwrap();
    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
