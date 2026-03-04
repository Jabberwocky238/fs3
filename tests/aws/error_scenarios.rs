use aws_sdk_s3::primitives::ByteStream;
use super::helpers::{create_aws_client, create_test_server};

#[tokio::test(flavor = "multi_thread")]
async fn error_scenarios_test() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);

    // nonexistent bucket
    let err = client.head_bucket().bucket("nonexistent").send().await;
    assert!(err.is_err());

    // nonexistent object
    let err = client.head_object().bucket("nonexistent").key("key").send().await;
    assert!(err.is_err());

    // create existing bucket
    client.create_bucket().bucket("test").send().await.unwrap();
    let err = client.create_bucket().bucket("test").send().await;
    assert!(err.is_err());

    // delete non-empty bucket
    client.put_object().bucket("test").key("file").body(ByteStream::from_static(b"data")).send().await.unwrap();
    let err = client.delete_bucket().bucket("test").send().await;
    assert!(err.is_err());

    // cleanup
    client.delete_object().bucket("test").key("file").send().await.unwrap();
    client.delete_bucket().bucket("test").send().await.unwrap();

    handle.abort();
}
