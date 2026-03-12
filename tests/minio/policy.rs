use minio::s3::builders::ObjectContent;
use minio::s3::types::S3Api;

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn policy_engine_test() {
    let (_addr, endpoint, _task) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "policy-test-bucket";

    // 创建桶
    client.create_bucket(bucket).send().await.unwrap();

    // 测试1: 设置允许所有人读取的桶策略
    let allow_policy = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": "*",
            "Action": ["s3:GetObject"],
            "Resource": ["arn:aws:s3:::policy-test-bucket/*"]
        }]
    }"#;
    client
        .put_bucket_policy(bucket)
        .config(allow_policy.to_string())
        .send()
        .await
        .unwrap();
    let got_policy = client.get_bucket_policy(bucket).send().await.unwrap();
    assert!(!got_policy.config.is_empty());

    // 测试2: 更新为多条语句的策略
    let multi_stmt_policy = r#"{
        "Version": "2012-10-17",
        "Statement": [
            {
                "Effect": "Allow",
                "Principal": "*",
                "Action": ["s3:GetObject", "s3:PutObject"],
                "Resource": ["arn:aws:s3:::policy-test-bucket/public/*"]
            },
            {
                "Effect": "Deny",
                "Principal": "*",
                "Action": ["s3:DeleteObject"],
                "Resource": ["arn:aws:s3:::policy-test-bucket/*"]
            }
        ]
    }"#;
    client
        .put_bucket_policy(bucket)
        .config(multi_stmt_policy.to_string())
        .send()
        .await
        .unwrap();
    let got_policy = client.get_bucket_policy(bucket).send().await.unwrap();
    assert!(got_policy.config.contains("PutObject"));
    assert!(got_policy.config.contains("DeleteObject"));

    // 测试3: 上传对象并验证策略生效
    let key = "public/test.txt";
    let data = b"test data";
    client
        .put_object_content(bucket, key, ObjectContent::from(data.as_ref()))
        .send()
        .await
        .unwrap();
    let resp = client.get_object(bucket, key).send().await.unwrap();
    let content = resp.content.to_segmented_bytes().await.unwrap().to_bytes();
    assert_eq!(content.as_ref(), data);

    // 测试4: 测试无效策略被拒绝
    let invalid_policy = r#"{"invalid json"#;
    let result = client
        .put_bucket_policy(bucket)
        .config(invalid_policy.to_string())
        .send()
        .await;
    assert!(result.is_err());

    // 测试5: 删除策略
    client.delete_bucket_policy(bucket).send().await.unwrap();
    let result = client.get_bucket_policy(bucket).send().await;
    assert!(result.is_err() || result.unwrap().config.is_empty());

    // 清理
    client.delete_object(bucket, key).send().await.unwrap();
    client.delete_bucket(bucket).send().await.unwrap();
}
