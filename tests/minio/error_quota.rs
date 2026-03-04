use crate::helpers::*;

#[tokio::test]
async fn test_quota_exceeded() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "test-object";
    let data = vec![0u8; 1024];
    client.put_object().bucket(&bucket).key(key).body(data.into()).send().await.unwrap();
}
