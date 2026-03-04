use super::helpers::*;
use minio::s3::types::S3Api;

#[tokio::test(flavor = "multi_thread")]
async fn test_put_bucket_website() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "website-bucket";
    client.create_bucket(bucket).send().await.unwrap();

    // MinIO website configuration uses different types
    // This test needs to be implemented based on MinIO SDK capabilities

    handle.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_bucket_website() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "website-bucket2";
    client.create_bucket(bucket).send().await.unwrap();

    // MinIO website configuration uses different types
    // This test needs to be implemented based on MinIO SDK capabilities

    handle.abort();
}
