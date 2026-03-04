use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn multipart_upload_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "multipart-bucket";
    let key = "large-file.bin";

    client.create_bucket(bucket).send().await.unwrap();

    // 创建分片上传
    let upload = client.create_multipart_upload(bucket, key).send().await.unwrap();
    assert!(!upload.upload_id.is_empty());

    // 中止上传
    client.abort_multipart_upload(bucket, key, &upload.upload_id).send().await.unwrap();

    // 清理
    client.delete_bucket(bucket).send().await.unwrap();

    handle.abort();
}
