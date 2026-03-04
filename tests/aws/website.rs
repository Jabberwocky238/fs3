use super::helpers::*;

#[tokio::test(flavor = "multi_thread")]
async fn test_put_bucket_website() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "website-bucket";
    client.create_bucket().bucket(bucket).send().await.unwrap();

    // AWS website configuration
    let website_config = aws_sdk_s3::types::WebsiteConfiguration::builder()
        .index_document(
            aws_sdk_s3::types::IndexDocument::builder()
                .suffix("index.html")
                .build()
                .unwrap()
        )
        .build();

    client.put_bucket_website()
        .bucket(bucket)
        .website_configuration(website_config)
        .send().await.unwrap();

    handle.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_bucket_website() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "website-bucket2";
    client.create_bucket().bucket(bucket).send().await.unwrap();

    let website_config = aws_sdk_s3::types::WebsiteConfiguration::builder()
        .index_document(
            aws_sdk_s3::types::IndexDocument::builder()
                .suffix("index.html")
                .build()
                .unwrap()
        )
        .build();

    client.put_bucket_website()
        .bucket(bucket)
        .website_configuration(website_config)
        .send().await.unwrap();

    let result = client.get_bucket_website().bucket(bucket).send().await.unwrap();
    assert_eq!(result.index_document().unwrap().suffix(), "index.html");

    handle.abort();
}
