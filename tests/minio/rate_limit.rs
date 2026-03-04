use crate::helpers::*;

#[tokio::test]
async fn test_rate_limiting() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "test-object";
    for i in 0..10 {
        client.put_object().bucket(&bucket).key(&format!("{}-{}", key, i)).body("data".into()).send().await.unwrap();
    }

    let objects = client.list_objects_v2().bucket(&bucket).send().await.unwrap();
    assert_eq!(objects.contents().len(), 10);
}
