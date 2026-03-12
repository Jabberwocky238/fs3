use super::helpers::*;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ServerSideEncryption;

#[tokio::test]
async fn test_put_object_sse_kms() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket().bucket(&bucket).send().await.unwrap();

    let key = "kms-encrypted";
    client
        .put_object()
        .bucket(&bucket)
        .key(key)
        .body(ByteStream::from_static(b"secret"))
        .server_side_encryption(ServerSideEncryption::AwsKms)
        .send()
        .await
        .unwrap();
}
