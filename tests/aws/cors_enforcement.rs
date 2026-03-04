use super::helpers::*;
use aws_sdk_s3::types::{CorsConfiguration, CorsRule};

#[tokio::test]
async fn test_cors_preflight() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let cors = CorsConfiguration::builder()
        .cors_rules(CorsRule::builder()
            .allowed_methods("GET")
            .allowed_methods("OPTIONS")
            .allowed_origins("https://example.com")
            .allowed_headers("*")
            .build().unwrap())
        .build();

    client.put_bucket_cors().bucket(&bucket).cors_configuration(cors).send().await.unwrap();
}
