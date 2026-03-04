use super::helpers::*;
use aws_sdk_s3::types::{RestoreRequest, GlacierJobParameters, Tier};
use aws_sdk_s3::primitives::ByteStream;

#[tokio::test]
async fn test_restore_object() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket().bucket(&bucket).send().await.unwrap();

    let key = "archived-object";
    client.put_object().bucket(&bucket).key(key).body(ByteStream::from_static(b"data")).send().await.unwrap();

    let restore = RestoreRequest::builder()
        .days(7)
        .glacier_job_parameters(GlacierJobParameters::builder().tier(Tier::Standard).build().unwrap())
        .build();

    let _ = client.restore_object().bucket(&bucket).key(key).restore_request(restore).send().await;
}
