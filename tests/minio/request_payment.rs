use crate::helpers::*;
use aws_sdk_s3::types::{RequestPaymentConfiguration, Payer};

#[tokio::test]
async fn test_put_bucket_request_payment() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let config = RequestPaymentConfiguration::builder()
        .payer(Payer::Requester)
        .build().unwrap();

    client.put_bucket_request_payment().bucket(&bucket).request_payment_configuration(config).send().await.unwrap();
}
