use super::helpers::*;

#[tokio::test]
async fn test_presigned_get_url() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "test-object";
    let data = "test data with unique content 12345";
    client.put_object().bucket(&bucket).key(key).body(data.into()).send().await.unwrap();

    let presigned = client.get_object().bucket(&bucket).key(key).presigned(std::time::Duration::from_secs(3600)).await.unwrap();

    let resp = reqwest::get(presigned.uri()).await.unwrap();
    assert!(resp.status().is_success(), "Presigned GET must return 200");
    assert_eq!(resp.text().await.unwrap(), data, "Presigned GET must return exact data");
}

#[tokio::test]
async fn test_presigned_put_url() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "test-upload";
    let data = "uploaded data via presigned PUT";
    let presigned = client.put_object().bucket(&bucket).key(key).presigned(std::time::Duration::from_secs(3600)).await.unwrap();

    let http_client = reqwest::Client::new();
    let put_resp = http_client.put(presigned.uri()).body(data).send().await.unwrap();
    assert!(put_resp.status().is_success(), "Presigned PUT must succeed");

    let obj = client.get_object().bucket(&bucket).key(key).send().await.unwrap();
    let retrieved = obj.body.collect().await.unwrap().to_vec();
    assert_eq!(String::from_utf8(retrieved).unwrap(), data, "Presigned PUT must upload exact data");
}
