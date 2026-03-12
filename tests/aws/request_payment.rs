use super::helpers::*;
use aws_sdk_s3::types::{Payer, RequestPaymentConfiguration};

#[tokio::test]
async fn test_put_bucket_request_payment() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket().bucket(&bucket).send().await.unwrap();

    let config = RequestPaymentConfiguration::builder()
        .payer(Payer::Requester)
        .build()
        .unwrap();

    client
        .put_bucket_request_payment()
        .bucket(&bucket)
        .request_payment_configuration(config)
        .send()
        .await
        .unwrap();
}
