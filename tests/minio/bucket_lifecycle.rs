use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn test_bucket_lifecycle() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "test-lifecycle";

    client.create_bucket(bucket).send().await.unwrap();

    // PUT
    client.put_bucket_lifecycle(bucket).send().await.unwrap();

    // GET
    let _lc = client.get_bucket_lifecycle(bucket).send().await.unwrap();

    // DELETE
    client.delete_bucket_lifecycle(bucket).send().await.unwrap();

    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
