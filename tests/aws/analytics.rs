use super::helpers::*;
use aws_sdk_s3::types::{AnalyticsConfiguration, StorageClassAnalysis};

#[tokio::test]
async fn test_put_bucket_analytics() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let config = AnalyticsConfiguration::builder()
        .id("analytics1")
        .storage_class_analysis(StorageClassAnalysis::builder().build())
        .build().unwrap();

    client.put_bucket_analytics_configuration().bucket(&bucket).id("analytics1").analytics_configuration(config).send().await.unwrap();
}
