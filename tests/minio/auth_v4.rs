use crate::helpers::*;

#[tokio::test]
async fn test_signature_v4_auth() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "test-auth";
    client.put_object().bucket(&bucket).key(key).body("data".into()).send().await.unwrap();

    let obj = client.get_object().bucket(&bucket).key(key).send().await.unwrap();
    assert!(obj.content_length().unwrap() > 0);
}
