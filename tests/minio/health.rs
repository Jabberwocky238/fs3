use super::helpers::*;

#[tokio::test]
async fn test_health_check() {
    let client = setup_client().await;

    let result = client.list_buckets().send().await;
    assert!(result.is_ok());
}
