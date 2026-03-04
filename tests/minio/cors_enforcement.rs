use crate::helpers::*;

#[tokio::test]
async fn test_cors_preflight() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    // TODO: implement CORS enforcement
}
