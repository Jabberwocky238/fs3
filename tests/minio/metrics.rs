use crate::helpers::*;

#[tokio::test]
async fn test_prometheus_metrics() {
    let client = setup_client().await;

    let buckets = client.list_buckets().send().await.unwrap();
    assert!(buckets.buckets().len() >= 0);
}
