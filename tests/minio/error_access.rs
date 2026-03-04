use crate::helpers::*;

#[tokio::test]
async fn test_access_denied() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    // TODO: implement access denied errors
}
