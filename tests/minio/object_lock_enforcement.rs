use crate::helpers::*;
use aws_sdk_s3::types::{ObjectLockRetention, ObjectLockRetentionMode};

#[tokio::test]
async fn test_object_lock_worm() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "locked-object";
    let data = "immutable data";
    client.put_object().bucket(&bucket).key(key).body(data.into()).send().await.unwrap();

    let retention = ObjectLockRetention::builder()
        .mode(ObjectLockRetentionMode::Compliance)
        .retain_until_date(aws_smithy_types::DateTime::from_secs(chrono::Utc::now().timestamp() + 86400))
        .build().unwrap();

    client.put_object_retention().bucket(&bucket).key(key).retention(retention).send().await.unwrap();

    let result = client.get_object_retention().bucket(&bucket).key(key).send().await.unwrap();
    assert_eq!(result.retention().unwrap().mode(), Some(&ObjectLockRetentionMode::Compliance), "Must be in Compliance mode");

    let delete_result = client.delete_object().bucket(&bucket).key(key).send().await;
    assert!(delete_result.is_err(), "Locked object must not be deletable");
}
