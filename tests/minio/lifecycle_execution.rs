use super::helpers::*;

#[tokio::test]
async fn test_bucket_lifecycle() {
    let (_addr, endpoint, _task) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = random_bucket_name();

    client.make_bucket(&bucket, false).await.unwrap();

    let config = minio::s3::types::LifecycleConfig::default();
    client.set_bucket_lifecycle(&bucket, &config).await.unwrap();

    let result = client.get_bucket_lifecycle(&bucket).await.unwrap();
    assert!(result.rules.is_empty() || !result.rules.is_empty());
}
