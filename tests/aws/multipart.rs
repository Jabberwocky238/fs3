use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::CompletedPart;
use super::helpers::{create_aws_client, create_test_server};

#[tokio::test(flavor = "multi_thread")]
async fn multipart_upload_test() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "multipart";
    let key = "f.bin";

    client.create_bucket().bucket(bucket).send().await.unwrap();

    // create multipart upload
    let upload = client.create_multipart_upload().bucket(bucket).key(key).send().await.unwrap();
    let upload_id = upload.upload_id().unwrap();

    // upload part 1
    let part1_data = b"part1data";
    let resp1 = client.upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .part_number(1)
        .body(ByteStream::from_static(part1_data))
        .send().await.unwrap();
    let part1 = CompletedPart::builder()
        .part_number(1)
        .e_tag(resp1.e_tag().unwrap())
        .build();

    // upload part 2
    let part2_data = b"part2data";
    let resp2 = client.upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .part_number(2)
        .body(ByteStream::from_static(part2_data))
        .send().await.unwrap();
    let part2 = CompletedPart::builder()
        .part_number(2)
        .e_tag(resp2.e_tag().unwrap())
        .build();

    // complete multipart upload
    client.complete_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .multipart_upload(
            aws_sdk_s3::types::CompletedMultipartUpload::builder()
                .parts(part1)
                .parts(part2)
                .build()
        )
        .send().await.unwrap();

    // verify object exists
    let head = client.head_object().bucket(bucket).key(key).send().await.unwrap();
    assert_eq!(head.content_length().unwrap(), (part1_data.len() + part2_data.len()) as i64);

    client.delete_object().bucket(bucket).key(key).send().await.unwrap();
    client.delete_bucket().bucket(bucket).send().await.unwrap();
    handle.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn multipart_abort_test() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "multipart-abort";
    let key = "abort.bin";

    client.create_bucket().bucket(bucket).send().await.unwrap();

    let upload = client.create_multipart_upload().bucket(bucket).key(key).send().await.unwrap();
    client.abort_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload.upload_id().unwrap())
        .send().await.unwrap();

    client.delete_bucket().bucket(bucket).send().await.unwrap();
    handle.abort();
}
