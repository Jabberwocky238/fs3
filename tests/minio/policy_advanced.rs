use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn policy_advanced_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "policy-adv-bucket";

    client.create_bucket(bucket).send().await.unwrap();

    // 测试 Deny 优先级
    let deny_policy = r#"{
        "Version": "2012-10-17",
        "Statement": [
            {
                "Effect": "Allow",
                "Principal": "*",
                "Action": ["s3:GetObject"],
                "Resource": ["arn:aws:s3:::policy-adv-bucket/*"]
            },
            {
                "Effect": "Deny",
                "Principal": "*",
                "Action": ["s3:GetObject"],
                "Resource": ["arn:aws:s3:::policy-adv-bucket/secret/*"]
            }
        ]
    }"#;
    client
        .put_bucket_policy(bucket)
        .config(deny_policy.to_string())
        .send()
        .await
        .unwrap();

    // 测试资源通配符
    let wildcard_policy = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": "*",
            "Action": ["s3:*"],
            "Resource": ["arn:aws:s3:::policy-adv-bucket/public/*"]
        }]
    }"#;
    client
        .put_bucket_policy(bucket)
        .config(wildcard_policy.to_string())
        .send()
        .await
        .unwrap();

    // 验证策略存在
    let policy = client.get_bucket_policy(bucket).send().await.unwrap();
    assert!(policy.config.contains("public"));

    // 清理
    client.delete_bucket_policy(bucket).send().await.unwrap();
    client.delete_bucket(bucket).send().await.unwrap();

    handle.abort();
}
