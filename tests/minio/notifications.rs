use super::helpers::*;

#[tokio::test]
async fn test_event_notification_webhook() {
    let (_addr, endpoint, _task) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = random_bucket_name();

    client.create_bucket(&bucket).send().await.unwrap();

    client.put_bucket_notification(&bucket).send().await.unwrap();

    let _result = client.get_bucket_notification(&bucket).send().await.unwrap();
}
