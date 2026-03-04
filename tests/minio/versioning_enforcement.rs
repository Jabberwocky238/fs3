use crate::helpers::*;
use aws_sdk_s3::types::{BucketVersioningStatus, VersioningConfiguration};

#[tokio::test]
async fn test_versioning_keeps_history() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let versioning = VersioningConfiguration::builder()
        .status(BucketVersioningStatus::Enabled)
        .build();

    client.put_bucket_versioning().bucket(&bucket).versioning_configuration(versioning).send().await.unwrap();

    let key = "versioned-object";
    client.put_object().bucket(&bucket).key(key).body("v1".into()).send().await.unwrap();
    client.put_object().bucket(&bucket).key(key).body("v2".into()).send().await.unwrap();
    client.put_object().bucket(&bucket).key(key).body("v3".into()).send().await.unwrap();

    let versions = client.list_object_versions().bucket(&bucket).send().await.unwrap();
    assert!(versions.versions().len() >= 3, "Must keep all 3 versions");

    let latest = client.get_object().bucket(&bucket).key(key).send().await.unwrap();
    let data = latest.body.collect().await.unwrap().to_vec();
    assert_eq!(String::from_utf8(data).unwrap(), "v3", "Latest version must be v3");
}
