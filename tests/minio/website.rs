use crate::helpers::*;
use aws_sdk_s3::types::{WebsiteConfiguration, IndexDocument, ErrorDocument};

#[tokio::test]
async fn test_put_bucket_website() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let website = WebsiteConfiguration::builder()
        .index_document(IndexDocument::builder().suffix("index.html").build().unwrap())
        .error_document(ErrorDocument::builder().key("error.html").build().unwrap())
        .build();

    client.put_bucket_website().bucket(&bucket).website_configuration(website).send().await.unwrap();

    let result = client.get_bucket_website().bucket(&bucket).send().await.unwrap();
    assert_eq!(result.index_document().unwrap().suffix(), "index.html", "Index must match");
    assert_eq!(result.error_document().unwrap().key(), "error.html", "Error page must match");
}

#[tokio::test]
async fn test_get_bucket_website() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let website = WebsiteConfiguration::builder()
        .index_document(IndexDocument::builder().suffix("home.html").build().unwrap())
        .build();

    client.put_bucket_website().bucket(&bucket).website_configuration(website).send().await.unwrap();
    let result = client.get_bucket_website().bucket(&bucket).send().await.unwrap();
    assert_eq!(result.index_document().unwrap().suffix(), "home.html", "Must return exact index");
}
