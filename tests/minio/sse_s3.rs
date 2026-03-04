use super::helpers::*;

#[tokio::test]
async fn test_bucket_encryption() {
    let (_addr, endpoint, _task) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = random_bucket_name();

    client.make_bucket(&bucket, false).await.unwrap();

    let config = minio::s3::types::SseConfig::s3();
    client.set_bucket_encryption(&bucket, &config).await.unwrap();

    let result = client.get_bucket_encryption(&bucket).await.unwrap();
    assert!(result.sse_algorithm.is_some());
}
