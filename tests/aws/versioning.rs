use super::helpers::{create_aws_client, create_test_server};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::BucketVersioningStatus;

#[tokio::test(flavor = "multi_thread")]
async fn versioning_test() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "version-bucket";
    let key = "versioned.txt";

    client.create_bucket().bucket(bucket).send().await.unwrap();

    // enable versioning
    client
        .put_bucket_versioning()
        .bucket(bucket)
        .versioning_configuration(
            aws_sdk_s3::types::VersioningConfiguration::builder()
                .status(BucketVersioningStatus::Enabled)
                .build(),
        )
        .send()
        .await
        .unwrap();

    // upload multiple versions
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from_static(b"v1"))
        .send()
        .await
        .unwrap();
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from_static(b"v2"))
        .send()
        .await
        .unwrap();
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from_static(b"v3"))
        .send()
        .await
        .unwrap();

    // get latest version
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .unwrap();
    let content = resp.body.collect().await.unwrap().to_vec();
    assert_eq!(content.as_slice(), b"v3");

    // delete object
    client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .unwrap();

    // cleanup
    client.delete_bucket().bucket(bucket).send().await.unwrap();

    handle.abort();
}
