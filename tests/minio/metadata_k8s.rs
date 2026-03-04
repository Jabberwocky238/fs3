use super::helpers::*;

#[tokio::test]
async fn test_k8s_configmap_metadata() {
    let client = setup_client().await;

    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "test-object";
    client.put_object().bucket(&bucket).key(key).body("data".into()).send().await.unwrap();

    let obj = client.head_object().bucket(&bucket).key(key).send().await.unwrap();
    assert!(obj.content_length().unwrap() > 0);
}
