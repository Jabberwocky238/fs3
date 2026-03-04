use crate::helpers::*;
use aws_sdk_s3::types::ServerSideEncryption;

#[tokio::test]
async fn test_put_object_sse_s3() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "encrypted-object";
    let data = "secret data with SSE-S3";
    client.put_object()
        .bucket(&bucket)
        .key(key)
        .body(data.into())
        .server_side_encryption(ServerSideEncryption::Aes256)
        .send().await.unwrap();

    let obj = client.get_object().bucket(&bucket).key(key).send().await.unwrap();
    assert_eq!(obj.server_side_encryption(), Some(&ServerSideEncryption::Aes256), "Must have SSE-S3 encryption");

    let body = obj.body.collect().await.unwrap().to_vec();
    assert_eq!(String::from_utf8(body).unwrap(), data, "Must decrypt and return original data");
}
