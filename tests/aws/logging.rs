use super::helpers::*;
use aws_sdk_s3::types::{BucketLoggingStatus, LoggingEnabled};

#[tokio::test]
async fn test_put_bucket_logging() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    let log_bucket = random_bucket_name();
    client.create_bucket().bucket(&bucket).send().await.unwrap();
    client
        .create_bucket()
        .bucket(&log_bucket)
        .send()
        .await
        .unwrap();

    let logging = BucketLoggingStatus::builder()
        .logging_enabled(
            LoggingEnabled::builder()
                .target_bucket(&log_bucket)
                .target_prefix("access-logs/")
                .build()
                .unwrap(),
        )
        .build();

    client
        .put_bucket_logging()
        .bucket(&bucket)
        .bucket_logging_status(logging)
        .send()
        .await
        .unwrap();

    let result = client
        .get_bucket_logging()
        .bucket(&bucket)
        .send()
        .await
        .unwrap();
    assert_eq!(
        result.logging_enabled().unwrap().target_bucket(),
        &log_bucket,
        "Target bucket must match"
    );
    assert_eq!(
        result.logging_enabled().unwrap().target_prefix(),
        "access-logs/",
        "Prefix must match"
    );
}
