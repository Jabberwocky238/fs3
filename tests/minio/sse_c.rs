use crate::helpers::*;

#[tokio::test]
async fn test_put_object_sse_c() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "encrypted-object";
    let encryption_key = "12345678901234567890123456789012";

    client.put_object()
        .bucket(&bucket)
        .key(key)
        .body("secret data".into())
        .sse_customer_algorithm("AES256")
        .sse_customer_key(encryption_key)
        .send().await.unwrap();

    let obj = client.get_object()
        .bucket(&bucket)
        .key(key)
        .sse_customer_algorithm("AES256")
        .sse_customer_key(encryption_key)
        .send().await.unwrap();

    assert_eq!(obj.sse_customer_algorithm(), Some("AES256"));
}
