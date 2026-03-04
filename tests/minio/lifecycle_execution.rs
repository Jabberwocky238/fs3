use crate::helpers::*;
use aws_sdk_s3::types::{BucketLifecycleConfiguration, LifecycleRule, LifecycleExpiration, ExpirationStatus};

#[tokio::test]
async fn test_lifecycle_expiration() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let lifecycle = BucketLifecycleConfiguration::builder()
        .rules(LifecycleRule::builder()
            .id("expire-old")
            .status(ExpirationStatus::Enabled)
            .expiration(LifecycleExpiration::builder().days(30).build())
            .build().unwrap())
        .build().unwrap();

    client.put_bucket_lifecycle_configuration().bucket(&bucket).lifecycle_configuration(lifecycle).send().await.unwrap();
}
