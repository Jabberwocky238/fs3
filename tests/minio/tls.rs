use crate::helpers::*;

#[tokio::test]
async fn test_tls_connection() {
    let client = setup_client().await;

    let result = client.list_buckets().send().await;
    assert!(result.is_ok());
}
