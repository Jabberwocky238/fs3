use aws_sdk_s3::primitives::ByteStream;
use super::helpers::{create_aws_client, create_test_server};

#[tokio::test(flavor = "multi_thread")]
async fn object_test() {
    let (_addr, endpoint, _task) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "obj-test-bucket";
    let key = "hello.txt";
    let data = b"hello world";

    // setup bucket
    client.create_bucket().bucket(bucket).send().await.unwrap();

    // put object
    client.put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from_static(data))
        .send().await.unwrap();

    // head object
    let head = client.head_object().bucket(bucket).key(key).send().await.unwrap();
    assert_eq!(head.content_length().unwrap(), data.len() as i64);

    // get object
    let resp = client.get_object().bucket(bucket).key(key).send().await.unwrap();
    let content = resp.body.collect().await.unwrap().to_vec();
    assert_eq!(content.as_slice(), data);

    // copy object
    let dest_key = "hello-copy.txt";
    client.copy_object()
        .bucket(bucket)
        .key(dest_key)
        .copy_source(format!("{}/{}", bucket, key))
        .send().await.unwrap();
    let head2 = client.head_object().bucket(bucket).key(dest_key).send().await.unwrap();
    assert_eq!(head2.content_length().unwrap(), data.len() as i64);

    // delete object
    client.delete_object().bucket(bucket).key(key).send().await.unwrap();

    // verify deleted
    assert!(client.head_object().bucket(bucket).key(key).send().await.is_err());

    // cleanup
    client.delete_object().bucket(bucket).key(dest_key).send().await.unwrap();
    client.delete_bucket().bucket(bucket).send().await.unwrap();
}
