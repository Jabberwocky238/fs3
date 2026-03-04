use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn multipart_upload_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "multipart";

    client.create_bucket(bucket).send().await.unwrap();

    let upload = client.create_multipart_upload(bucket, "f.bin").send().await.unwrap();
    assert!(!upload.upload_id.is_empty());

    client.abort_multipart_upload(bucket, "f.bin", &upload.upload_id).send().await.unwrap();

    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
