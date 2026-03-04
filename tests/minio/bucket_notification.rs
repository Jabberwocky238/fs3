use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn test_bucket_notification() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "test-notification";

    client.create_bucket(bucket).send().await.unwrap();

    // PUT
    client.put_bucket_notification(bucket).send().await.unwrap();

    // GET
    let _n = client.get_bucket_notification(bucket).send().await.unwrap();

    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
