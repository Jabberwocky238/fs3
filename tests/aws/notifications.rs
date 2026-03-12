use super::helpers::*;
use aws_sdk_s3::types::{Event, NotificationConfiguration, QueueConfiguration};

#[tokio::test]
async fn test_event_notification_webhook() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket().bucket(&bucket).send().await.unwrap();

    let config = NotificationConfiguration::builder()
        .queue_configurations(
            QueueConfiguration::builder()
                .queue_arn("arn:aws:sqs:us-east-1:123456789012:test-queue")
                .events(Event::S3ObjectCreatedPut)
                .events(Event::S3ObjectCreatedPost)
                .build()
                .unwrap(),
        )
        .build();

    client
        .put_bucket_notification_configuration()
        .bucket(&bucket)
        .notification_configuration(config)
        .send()
        .await
        .unwrap();

    let result = client
        .get_bucket_notification_configuration()
        .bucket(&bucket)
        .send()
        .await
        .unwrap();
    assert!(
        !result.queue_configurations().is_empty(),
        "Must have queue config"
    );
    assert_eq!(
        result.queue_configurations()[0].events().len(),
        2,
        "Must have 2 events"
    );
}
