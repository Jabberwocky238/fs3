use crate::helpers::*;
use aws_sdk_s3::types::ServerSideEncryption;

#[tokio::test]
async fn test_put_object_sse_s3() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "encrypted-object";
    client.put_object()
        .bucket(&bucket)
        .key(key)
        .body("secret data".into())
        .server_side_encryption(ServerSideEncryption::Aes256)
        .send().await.unwrap();

    let obj = client.get_object().bucket(&bucket).key(key).send().await.unwrap();
    assert_eq!(obj.server_side_encryption(), Some(&ServerSideEncryption::Aes256));
}
