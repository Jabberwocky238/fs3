use minio::s3::types::S3Api;
use minio::s3::types::PartInfo;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn multipart_upload_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "multipart";
    let key = "f.bin";

    client.create_bucket(bucket).send().await.unwrap();

    // 创建multipart upload
    let upload = client.create_multipart_upload(bucket, key).send().await.unwrap();
    assert!(!upload.upload_id.is_empty());

    // 上传part 1
    let part1_data = b"part1data";
    let resp1 = client.upload_part(bucket, key, &upload.upload_id, 1, bytes::Bytes::from_static(part1_data).into()).send().await.unwrap();
    let part1 = PartInfo { number: 1, size: part1_data.len() as u64, etag: resp1.etag };

    // 上传part 2
    let part2_data = b"part2data";
    let resp2 = client.upload_part(bucket, key, &upload.upload_id, 2, bytes::Bytes::from_static(part2_data).into()).send().await.unwrap();
    let part2 = PartInfo { number: 2, size: part2_data.len() as u64, etag: resp2.etag };

    // 完成multipart upload
    client.complete_multipart_upload(bucket, key, &upload.upload_id, vec![part1, part2]).send().await.unwrap();

    // 验证对象存在
    let obj = client.stat_object(bucket, key).send().await.unwrap();
    assert_eq!(obj.size, (part1_data.len() + part2_data.len()) as u64);

    client.delete_object(bucket, key).send().await.unwrap();
    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn multipart_abort_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "multipart-abort";
    let key = "abort.bin";

    client.create_bucket(bucket).send().await.unwrap();

    let upload = client.create_multipart_upload(bucket, key).send().await.unwrap();
    client.abort_multipart_upload(bucket, key, &upload.upload_id).send().await.unwrap();

    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
