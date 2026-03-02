use minio::s3::builders::{CopySource, ObjectContent};
use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn object_test() {
    let (_addr, endpoint, _task) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "obj-test-bucket";
    let key = "hello.txt";
    let data = b"hello world";

    // setup bucket
    client.create_bucket(bucket).send().await.unwrap();

    // put object
    client
        .put_object_content(bucket, key, ObjectContent::from(data.as_ref()))
        .send().await.unwrap();

    // stat object (head)
    let stat = client.stat_object(bucket, key).send().await.unwrap();
    assert_eq!(stat.size, data.len() as u64);

    // get object
    let resp = client.get_object(bucket, key).send().await.unwrap();
    let content = resp.content.to_segmented_bytes().await.unwrap().to_bytes();
    assert_eq!(content.as_ref(), data);

    // copy object
    let dest_key = "hello-copy.txt";
    let src = CopySource::new(bucket, key).unwrap();
    client.copy_object(bucket, dest_key).source(src).send().await.unwrap();
    let stat2 = client.stat_object(bucket, dest_key).send().await.unwrap();
    assert_eq!(stat2.size, data.len() as u64);

    // delete object
    client.delete_object(bucket, key).send().await.unwrap();

    // verify deleted - stat should fail
    let err = client.stat_object(bucket, key).send().await;
    assert!(err.is_err());

    // cleanup
    client.delete_object(bucket, dest_key).send().await.unwrap();
    client.delete_bucket(bucket).send().await.unwrap();
}
