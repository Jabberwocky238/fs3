use super::helpers::*;
use aws_sdk_s3::types::{
    BucketLifecycleConfiguration, ExpirationStatus, LifecycleExpiration, LifecycleRule,
    LifecycleRuleFilter,
};

#[tokio::test]
async fn test_lifecycle_expiration() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket().bucket(&bucket).send().await.unwrap();

    let lifecycle = BucketLifecycleConfiguration::builder()
        .rules(
            LifecycleRule::builder()
                .id("expire-old")
                .status(ExpirationStatus::Enabled)
                .filter(LifecycleRuleFilter::builder().prefix("logs/").build())
                .expiration(LifecycleExpiration::builder().days(30).build())
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    client
        .put_bucket_lifecycle_configuration()
        .bucket(&bucket)
        .lifecycle_configuration(lifecycle)
        .send()
        .await
        .unwrap();

    let result = client
        .get_bucket_lifecycle_configuration()
        .bucket(&bucket)
        .send()
        .await
        .unwrap();
    assert_eq!(result.rules().len(), 1, "Must have 1 lifecycle rule");
    assert_eq!(
        result.rules()[0].id(),
        Some("expire-old"),
        "Rule ID must match"
    );
}
