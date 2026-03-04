use super::helpers::*;
use minio::s3::types::S3Api;

#[tokio::test]
async fn test_bucket_encryption() {
    let (_addr, endpoint, _task) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = random_bucket_name();

    client.create_bucket(&bucket).send().await.unwrap();

    client.put_bucket_encryption(&bucket).send().await.unwrap();

    let _result = client.get_bucket_encryption(&bucket).send().await.unwrap();
}
