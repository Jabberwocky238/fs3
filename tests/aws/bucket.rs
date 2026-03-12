use super::helpers::{create_aws_client, create_test_server};
use aws_sdk_s3::types::{BucketVersioningStatus, Tag};

#[tokio::test(flavor = "multi_thread")]
async fn bucket_test() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "test-bucket";

    // create bucket
    client.create_bucket().bucket(bucket).send().await.unwrap();

    // head bucket (exists check)
    client.head_bucket().bucket(bucket).send().await.unwrap();

    // list buckets
    let list = client.list_buckets().send().await.unwrap();
    assert!(list.buckets().iter().any(|b| b.name().unwrap() == bucket));

    // get bucket location
    let _location = client
        .get_bucket_location()
        .bucket(bucket)
        .send()
        .await
        .unwrap();

    // put/get/delete bucket policy
    let policy = r#"{"Version":"2012-10-17","Statement":[]}"#;
    client
        .put_bucket_policy()
        .bucket(bucket)
        .policy(policy)
        .send()
        .await
        .unwrap();
    let got_policy = client
        .get_bucket_policy()
        .bucket(bucket)
        .send()
        .await
        .unwrap();
    assert!(!got_policy.policy().unwrap().is_empty());
    client
        .delete_bucket_policy()
        .bucket(bucket)
        .send()
        .await
        .unwrap();

    // put/get/delete bucket tagging
    let tag = Tag::builder().key("env").value("test").build().unwrap();
    client
        .put_bucket_tagging()
        .bucket(bucket)
        .tagging(
            aws_sdk_s3::types::Tagging::builder()
                .tag_set(tag)
                .build()
                .unwrap(),
        )
        .send()
        .await
        .unwrap();
    let got_tags = client
        .get_bucket_tagging()
        .bucket(bucket)
        .send()
        .await
        .unwrap();
    assert!(!got_tags.tag_set().is_empty());
    client
        .delete_bucket_tagging()
        .bucket(bucket)
        .send()
        .await
        .unwrap();

    // put/get bucket versioning
    client
        .put_bucket_versioning()
        .bucket(bucket)
        .versioning_configuration(
            aws_sdk_s3::types::VersioningConfiguration::builder()
                .status(BucketVersioningStatus::Enabled)
                .build(),
        )
        .send()
        .await
        .unwrap();
    let ver = client
        .get_bucket_versioning()
        .bucket(bucket)
        .send()
        .await
        .unwrap();
    assert!(ver.status().is_some());

    // delete bucket encryption
    client
        .delete_bucket_encryption()
        .bucket(bucket)
        .send()
        .await
        .unwrap();

    // delete bucket lifecycle
    client
        .delete_bucket_lifecycle()
        .bucket(bucket)
        .send()
        .await
        .unwrap();

    // delete bucket replication
    client
        .delete_bucket_replication()
        .bucket(bucket)
        .send()
        .await
        .unwrap();

    // get bucket notification
    let _notif = client
        .get_bucket_notification_configuration()
        .bucket(bucket)
        .send()
        .await
        .unwrap();

    // delete bucket
    client.delete_bucket().bucket(bucket).send().await.unwrap();

    // verify deleted
    assert!(client.head_bucket().bucket(bucket).send().await.is_err());

    handle.abort();
}
