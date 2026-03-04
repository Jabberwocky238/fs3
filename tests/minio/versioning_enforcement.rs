use super::helpers::*;

#[tokio::test]
async fn test_bucket_versioning() {
    let (_addr, endpoint, _task) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = random_bucket_name();

    client.make_bucket(&bucket, false).await.unwrap();

    client.enable_versioning(&bucket).await.unwrap();

    let result = client.is_versioning_enabled(&bucket).await.unwrap();
    assert!(result);
}
