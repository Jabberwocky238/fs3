use minio::s3::types::S3Api;
use minio::s3::builders::ObjectContent;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn multipart_upload_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "mp-bucket";

    client.create_bucket(bucket).send().await.unwrap();

    // 完整的分片上传流程
    let key1 = "complete.bin";
    let upload = client.create_multipart_upload(bucket, key1).send().await.unwrap();
    let uid = &upload.upload_id;

    let data1 = bytes::Bytes::from("part1");
    let data2 = bytes::Bytes::from("part2");
    let p1 = client.upload_part(bucket, key1, uid, 1, data1.into()).send().await.unwrap();
    let p2 = client.upload_part(bucket, key1, uid, 2, data2.into()).send().await.unwrap();

    let parts = client.list_parts(bucket, key1, uid).send().await.unwrap();
    assert_eq!(parts.parts.len(), 2);

    let cparts = vec![
        minio::s3::types::Part::new(1, &p1.etag.unwrap()),
        minio::s3::types::Part::new(2, &p2.etag.unwrap()),
    ];
    client.complete_multipart_upload(bucket, key1, uid, cparts).send().await.unwrap();

    let obj = client.stat_object(bucket, key1).send().await.unwrap();
    assert_eq!(obj.size, 10);

    // 测试中止上传
    let key2 = "abort.bin";
    let upload2 = client.create_multipart_upload(bucket, key2).send().await.unwrap();
    client.abort_multipart_upload(bucket, key2, &upload2.upload_id).send().await.unwrap();

    // 清理
    client.delete_object(bucket, key1).send().await.unwrap();
    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
