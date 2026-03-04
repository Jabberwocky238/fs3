use crate::helpers::*;

#[tokio::test]
async fn test_presigned_get_url() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    // TODO: implement pre-signed GET
}

#[tokio::test]
async fn test_presigned_put_url() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    // TODO: implement pre-signed PUT
}
