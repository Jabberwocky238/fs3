use crate::helpers::*;
use aws_sdk_s3::types::{BucketLifecycleConfiguration, LifecycleRule, LifecycleExpiration, ExpirationStatus, LifecycleRuleFilter};

#[tokio::test]
async fn test_lifecycle_expiration() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let lifecycle = BucketLifecycleConfiguration::builder()
        .rules(LifecycleRule::builder()
            .id("expire-old")
            .status(ExpirationStatus::Enabled)
            .filter(LifecycleRuleFilter::Prefix("logs/".to_string()))
            .expiration(LifecycleExpiration::builder().days(30).build())
            .build().unwrap())
        .build().unwrap();

    client.put_bucket_lifecycle_configuration().bucket(&bucket).lifecycle_configuration(lifecycle).send().await.unwrap();

    let result = client.get_bucket_lifecycle_configuration().bucket(&bucket).send().await.unwrap();
    assert_eq!(result.rules().len(), 1, "Must have 1 lifecycle rule");
    assert_eq!(result.rules()[0].id(), Some("expire-old"), "Rule ID must match");
}
