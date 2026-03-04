use minio::s3::types::S3Api;
use std::collections::HashMap;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn test_bucket_tagging() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "test-tagging";

    client.create_bucket(bucket).send().await.unwrap();

    // PUT bucket tagging
    let mut tags = HashMap::new();
    tags.insert("env".to_string(), "test".to_string());
    tags.insert("team".to_string(), "dev".to_string());
    client.put_bucket_tagging(bucket).tags(tags.clone()).send().await.unwrap();

    // GET bucket tagging
    let got_tags = client.get_bucket_tagging(bucket).send().await.unwrap();
    assert_eq!(got_tags.tags.len(), 2);
    assert_eq!(got_tags.tags.get("env"), Some(&"test".to_string()));
    assert_eq!(got_tags.tags.get("team"), Some(&"dev".to_string()));

    // DELETE bucket tagging
    client.delete_bucket_tagging(bucket).send().await.unwrap();

    // Verify deleted
    let result = client.get_bucket_tagging(bucket).send().await.unwrap();
    assert!(result.tags.is_empty());

    client.delete_bucket(bucket).send().await.unwrap();
    handle.abort();
}
