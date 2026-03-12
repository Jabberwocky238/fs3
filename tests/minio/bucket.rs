use std::collections::HashMap;

use minio::s3::builders::VersioningStatus;
use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn bucket_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "test-bucket";

    // create bucket
    client.create_bucket(bucket).send().await.unwrap();

    // bucket exists
    let exists = client.bucket_exists(bucket).send().await.unwrap();
    assert!(exists.exists);

    // list buckets
    let list = client.list_buckets().send().await.unwrap();
    assert!(list.buckets.iter().any(|b| b.name == bucket));

    // get region
    let region = client.get_region(bucket).send().await.unwrap();
    assert!(!region.region.is_empty());

    // put/get/delete bucket policy
    let policy = r#"{"Version":"2012-10-17","Statement":[]}"#;
    client
        .put_bucket_policy(bucket)
        .config(policy.to_string())
        .send()
        .await
        .unwrap();
    let got_policy = client.get_bucket_policy(bucket).send().await.unwrap();
    assert!(!got_policy.config.is_empty());
    client.delete_bucket_policy(bucket).send().await.unwrap();

    // put/get/delete bucket tagging
    let mut tags = HashMap::new();
    tags.insert("env".to_string(), "test".to_string());
    client
        .put_bucket_tagging(bucket)
        .tags(tags)
        .send()
        .await
        .unwrap();
    let got_tags = client.get_bucket_tagging(bucket).send().await.unwrap();
    assert!(!got_tags.tags.is_empty());
    client.delete_bucket_tagging(bucket).send().await.unwrap();

    // put/get bucket versioning
    client
        .put_bucket_versioning(bucket)
        .versioning_status(VersioningStatus::Enabled)
        .send()
        .await
        .unwrap();
    let ver = client.get_bucket_versioning(bucket).send().await.unwrap();
    assert!(ver.status.is_some());

    // get/delete bucket encryption
    client
        .delete_bucket_encryption(bucket)
        .send()
        .await
        .unwrap();

    // get/delete bucket lifecycle
    client.delete_bucket_lifecycle(bucket).send().await.unwrap();

    // get/delete bucket replication
    client
        .delete_bucket_replication(bucket)
        .send()
        .await
        .unwrap();

    // get bucket notification
    let _notif = client.get_bucket_notification(bucket).send().await.unwrap();

    // delete bucket
    client.delete_bucket(bucket).send().await.unwrap();

    // verify deleted
    let exists = client.bucket_exists(bucket).send().await.unwrap();
    assert!(!exists.exists);

    handle.abort();
}
