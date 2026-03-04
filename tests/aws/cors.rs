use super::helpers::*;
use aws_sdk_s3::types::{CorsConfiguration, CorsRule};

#[tokio::test]
async fn test_put_bucket_cors() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket().bucket(&bucket).send().await.unwrap();

    let cors = CorsConfiguration::builder()
        .cors_rules(CorsRule::builder()
            .allowed_methods("GET")
            .allowed_origins("https://example.com")
            .build().unwrap())
        .build().unwrap();

    client.put_bucket_cors().bucket(&bucket).cors_configuration(cors).send().await.unwrap();

    let result = client.get_bucket_cors().bucket(&bucket).send().await.unwrap();
    assert_eq!(result.cors_rules().len(), 1, "Must have exactly 1 CORS rule");
    assert_eq!(result.cors_rules()[0].allowed_origins(), &["https://example.com"], "Must match origin");
}

#[tokio::test]
async fn test_get_bucket_cors() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket().bucket(&bucket).send().await.unwrap();

    let cors = CorsConfiguration::builder()
        .cors_rules(CorsRule::builder()
            .allowed_methods("GET")
            .allowed_methods("POST")
            .allowed_origins("*")
            .build().unwrap())
        .build().unwrap();

    client.put_bucket_cors().bucket(&bucket).cors_configuration(cors).send().await.unwrap();
    let result = client.get_bucket_cors().bucket(&bucket).send().await.unwrap();
    assert_eq!(result.cors_rules()[0].allowed_methods().len(), 2, "Must have 2 methods");
}

#[tokio::test]
async fn test_delete_bucket_cors() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket().bucket(&bucket).send().await.unwrap();

    let cors = CorsConfiguration::builder()
        .cors_rules(CorsRule::builder()
            .allowed_methods("GET")
            .allowed_origins("*")
            .build().unwrap())
        .build().unwrap();

    client.put_bucket_cors().bucket(&bucket).cors_configuration(cors).send().await.unwrap();
    client.delete_bucket_cors().bucket(&bucket).send().await.unwrap();

    let result = client.get_bucket_cors().bucket(&bucket).send().await;
    assert!(result.is_err(), "CORS must be deleted");
}
