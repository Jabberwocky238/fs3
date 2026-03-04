use crate::helpers::*;

#[tokio::test]
async fn test_put_object_sse_s3() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    // TODO: implement SSE-S3 encryption
}
