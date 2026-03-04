use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ObjectLockLegalHold;
use super::helpers::{create_aws_client, create_test_server};

#[tokio::test(flavor = "multi_thread")]
async fn object_lock_test() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "locktest";
    let key = "locked.txt";

    client.create_bucket().bucket(bucket).send().await.unwrap();
    client.put_object().bucket(bucket).key(key).body(ByteStream::from_static(b"data")).send().await.unwrap();

    // Legal Hold
    let _ = client.put_object_legal_hold()
        .bucket(bucket)
        .key(key)
        .legal_hold(ObjectLockLegalHold::builder().status(aws_sdk_s3::types::ObjectLockLegalHoldStatus::On).build())
        .send().await;

    client.delete_object().bucket(bucket).key(key).send().await.unwrap();
    client.delete_bucket().bucket(bucket).send().await.unwrap();
    handle.abort();
}
