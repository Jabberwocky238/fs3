use crate::helpers::*;

#[tokio::test]
async fn test_put_bucket_cors() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket, None).await.unwrap();
    // TODO: implement CORS config
}

#[tokio::test]
async fn test_get_bucket_cors() {
    // TODO: implement
}

#[tokio::test]
async fn test_delete_bucket_cors() {
    // TODO: implement
}
