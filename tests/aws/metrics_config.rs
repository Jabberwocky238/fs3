use super::helpers::*;
use aws_sdk_s3::types::{MetricsConfiguration, MetricsFilter};

#[tokio::test]
async fn test_put_bucket_metrics() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket().bucket(&bucket).send().await.unwrap();

    let config = MetricsConfiguration::builder()
        .id("metrics1")
        .build().unwrap();

    client.put_bucket_metrics_configuration().bucket(&bucket).id("metrics1").metrics_configuration(config).send().await.unwrap();
}
