use super::helpers::{create_aws_client, create_test_server};

#[tokio::test(flavor = "multi_thread")]
async fn bucket_config_test() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "config";

    client.create_bucket().bucket(bucket).send().await.unwrap();

    // Encryption
    client
        .delete_bucket_encryption()
        .bucket(bucket)
        .send()
        .await
        .unwrap();

    // Lifecycle
    client
        .delete_bucket_lifecycle()
        .bucket(bucket)
        .send()
        .await
        .unwrap();

    // Replication
    client
        .delete_bucket_replication()
        .bucket(bucket)
        .send()
        .await
        .unwrap();

    // Notification
    let _n = client
        .get_bucket_notification_configuration()
        .bucket(bucket)
        .send()
        .await
        .unwrap();

    client.delete_bucket().bucket(bucket).send().await.unwrap();
    handle.abort();
}
