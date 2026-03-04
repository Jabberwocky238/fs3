use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn bucket_config_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "cfg";

    client.create_bucket(bucket).send().await.unwrap();

    // Encryption
    client.delete_bucket_encryption(bucket).send().await.unwrap();

    // Lifecycle
    client.delete_bucket_lifecycle(bucket).send().await.unwrap();

    // Replication
    client.delete_bucket_replication(bucket).send().await.unwrap();

    // Notification
    let _n = client.get_bucket_notification(bucket).send().await.unwrap();

    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
