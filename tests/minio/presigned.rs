use crate::helpers::*;

#[tokio::test]
async fn test_presigned_get_url() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "test-object";
    client.put_object().bucket(&bucket).key(key).body("test data".into()).send().await.unwrap();

    let presigned = client.get_object().bucket(&bucket).key(key).presigned(std::time::Duration::from_secs(3600)).await.unwrap();

    let resp = reqwest::get(presigned.uri()).await.unwrap();
    assert_eq!(resp.text().await.unwrap(), "test data");
}

#[tokio::test]
async fn test_presigned_put_url() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "test-upload";
    let presigned = client.put_object().bucket(&bucket).key(key).presigned(std::time::Duration::from_secs(3600)).await.unwrap();

    let http_client = reqwest::Client::new();
    http_client.put(presigned.uri()).body("uploaded data").send().await.unwrap();

    let obj = client.get_object().bucket(&bucket).key(key).send().await.unwrap();
    let data = obj.body.collect().await.unwrap().to_vec();
    assert_eq!(String::from_utf8(data).unwrap(), "uploaded data");
}
