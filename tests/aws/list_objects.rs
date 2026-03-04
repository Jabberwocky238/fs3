use aws_sdk_s3::primitives::ByteStream;
use super::helpers::{create_aws_client, create_test_server};

#[tokio::test(flavor = "multi_thread")]
async fn list_objects_test() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "list-bucket";

    client.create_bucket().bucket(bucket).send().await.unwrap();

    // create test objects
    client.put_object().bucket(bucket).key("a/1.txt").body(ByteStream::from_static(b"test")).send().await.unwrap();
    client.put_object().bucket(bucket).key("a/2.txt").body(ByteStream::from_static(b"test")).send().await.unwrap();
    client.put_object().bucket(bucket).key("b/1.txt").body(ByteStream::from_static(b"test")).send().await.unwrap();
    client.put_object().bucket(bucket).key("c.txt").body(ByteStream::from_static(b"test")).send().await.unwrap();

    // list all objects
    let mut count = 0;
    let mut continuation_token: Option<String> = None;
    loop {
        let mut req = client.list_objects_v2().bucket(bucket);
        if let Some(token) = continuation_token {
            req = req.continuation_token(token);
        }
        let resp = req.send().await.unwrap();
        count += resp.contents().len();
        if !resp.is_truncated().unwrap_or(false) {
            break;
        }
        continuation_token = resp.next_continuation_token().map(|s| s.to_string());
    }
    assert_eq!(count, 4);

    // cleanup
    client.delete_object().bucket(bucket).key("a/1.txt").send().await.unwrap();
    client.delete_object().bucket(bucket).key("a/2.txt").send().await.unwrap();
    client.delete_object().bucket(bucket).key("b/1.txt").send().await.unwrap();
    client.delete_object().bucket(bucket).key("c.txt").send().await.unwrap();
    client.delete_bucket().bucket(bucket).send().await.unwrap();

    handle.abort();
}
