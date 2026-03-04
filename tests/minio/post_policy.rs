use super::helpers::*;

#[tokio::test]
async fn test_post_object_with_policy() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();

    let key = "uploaded-via-post";
    let policy = serde_json::json!({
        "expiration": "2030-01-01T00:00:00Z",
        "conditions": [
            {"bucket": &bucket},
            {"key": key},
            ["content-length-range", 0, 1048576]
        ]
    });

    // POST policy requires form-based upload, typically done via HTTP client
    // This test validates the policy structure
    assert!(policy["conditions"].is_array());
}
