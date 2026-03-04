use super::helpers::*;
use aws_sdk_s3::types::ServerSideEncryption;

#[tokio::test]
async fn test_put_object_sse_kms() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "kms-encrypted";
    client.put_object()
        .bucket(&bucket)
        .key(key)
        .body("secret".into())
        .server_side_encryption(ServerSideEncryption::AwsKms)
        .send().await.unwrap();
}
