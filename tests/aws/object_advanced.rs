use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::Tag;
use super::helpers::{create_aws_client, create_test_server};

#[tokio::test(flavor = "multi_thread")]
async fn object_advanced_test() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "advanced-bucket";
    let key = "test.txt";
    let data = b"0123456789abcdefghij";

    client.create_bucket().bucket(bucket).send().await.unwrap();

    // upload object
    client.put_object().bucket(bucket).key(key).body(ByteStream::from_static(data)).send().await.unwrap();

    // object tagging
    let tag = Tag::builder().key("env").value("test").build().unwrap();
    client.put_object_tagging().bucket(bucket).key(key).tagging(
        aws_sdk_s3::types::Tagging::builder().tag_set(tag).build().unwrap()
    ).send().await.unwrap();
    let got_tags = client.get_object_tagging().bucket(bucket).key(key).send().await.unwrap();
    assert!(!got_tags.tag_set().is_empty());

    // delete tagging
    client.delete_object_tagging().bucket(bucket).key(key).send().await.unwrap();

    // cleanup
    client.delete_object().bucket(bucket).key(key).send().await.unwrap();
    client.delete_bucket().bucket(bucket).send().await.unwrap();

    handle.abort();
}
