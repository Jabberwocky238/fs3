use crate::helpers::*;

#[tokio::test]
async fn test_minio_gateway() {
    let client = setup_client().await;

    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let buckets = client.list_buckets().send().await.unwrap();
    assert!(buckets.buckets().iter().any(|b| b.name() == Some(&bucket)));
}
