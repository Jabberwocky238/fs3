use aws_sdk_s3::primitives::ByteStream;
use super::helpers::{create_aws_client, create_test_server};

#[tokio::test(flavor = "multi_thread")]
async fn object_features_test() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "features";
    let key = "test.txt";
    let data = b"0123456789abcdefghij";

    client.create_bucket().bucket(bucket).send().await.unwrap();
    client.put_object().bucket(bucket).key(key).body(ByteStream::from_static(data)).send().await.unwrap();

    // basic read
    let resp = client.get_object().bucket(bucket).key(key).send().await.unwrap();
    let content = resp.body.collect().await.unwrap().to_vec();
    assert_eq!(content.len(), 20);

    client.delete_object().bucket(bucket).key(key).send().await.unwrap();
    client.delete_bucket().bucket(bucket).send().await.unwrap();
    handle.abort();
}
