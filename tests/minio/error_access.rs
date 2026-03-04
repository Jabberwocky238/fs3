use crate::helpers::*;

#[tokio::test]
async fn test_access_denied() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let result = client.get_object().bucket(&bucket).key("nonexistent").send().await;
    assert!(result.is_err(), "Must return error for nonexistent object");

    let result2 = client.get_object().bucket("nonexistent-bucket").key("key").send().await;
    assert!(result2.is_err(), "Must return error for nonexistent bucket");
}
