use super::helpers::*;
use aws_sdk_s3::types::{AccelerateConfiguration, BucketAccelerateStatus};

#[tokio::test]
async fn test_put_bucket_accelerate() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let config = AccelerateConfiguration::builder()
        .status(BucketAccelerateStatus::Enabled)
        .build();

    client.put_bucket_accelerate_configuration().bucket(&bucket).accelerate_configuration(config).send().await.unwrap();
}
