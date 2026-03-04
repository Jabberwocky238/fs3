use super::helpers::*;

#[tokio::test]
async fn test_liveness_check() {
    let client = setup_client().await;
    let resp = client.get("/minio/health/live").send().await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_readiness_check() {
    let client = setup_client().await;
    let resp = client.get("/minio/health/ready").send().await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_cluster_check() {
    let client = setup_client().await;
    let resp = client.get("/minio/health/cluster").send().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.headers().contains_key("x-minio-write-quorum"));
}

#[tokio::test]
async fn test_cluster_read_check() {
    let client = setup_client().await;
    let resp = client.get("/minio/health/cluster/read").send().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.headers().contains_key("x-minio-read-quorum"));
}
