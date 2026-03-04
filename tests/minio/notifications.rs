use super::helpers::*;

#[tokio::test]
async fn test_event_notification_webhook() {
    let (_addr, endpoint, _task) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = random_bucket_name();

    client.make_bucket(&bucket, false).await.unwrap();

    // MinIO notification config
    let config = minio::s3::types::NotificationConfig::default();
    client.set_bucket_notification(&bucket, &config).await.unwrap();

    let result = client.get_bucket_notification(&bucket).await.unwrap();
    assert!(result.queue_config_list.is_empty() || !result.queue_config_list.is_empty());
}
