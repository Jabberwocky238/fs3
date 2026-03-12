use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn test_bucket_encryption() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "test-encryption";

    client.create_bucket(bucket).send().await.unwrap();

    // PUT
    client.put_bucket_encryption(bucket).send().await.unwrap();

    // GET
    let _enc = client.get_bucket_encryption(bucket).send().await.unwrap();

    // DELETE
    client
        .delete_bucket_encryption(bucket)
        .send()
        .await
        .unwrap();

    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
