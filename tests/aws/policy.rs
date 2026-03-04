use aws_sdk_s3::primitives::ByteStream;
use super::helpers::{create_aws_client, create_test_server};

#[tokio::test(flavor = "multi_thread")]
async fn policy_engine_test() {
    let (_addr, endpoint, _task) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "policy-test-bucket";

    client.create_bucket().bucket(bucket).send().await.unwrap();

    // test 1: allow public read
    let allow_policy = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": "*",
            "Action": ["s3:GetObject"],
            "Resource": ["arn:aws:s3:::policy-test-bucket/*"]
        }]
    }"#;
    client.put_bucket_policy().bucket(bucket).policy(allow_policy).send().await.unwrap();
    let got_policy = client.get_bucket_policy().bucket(bucket).send().await.unwrap();
    assert!(!got_policy.policy().unwrap().is_empty());

    // test 2: multi-statement policy
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
    client.put_bucket_policy().bucket(bucket).policy(multi_stmt_policy).send().await.unwrap();
    let got_policy = client.get_bucket_policy().bucket(bucket).send().await.unwrap();
    assert!(got_policy.policy().unwrap().contains("PutObject"));
    assert!(got_policy.policy().unwrap().contains("DeleteObject"));

    // test 3: upload object and verify policy
    let key = "public/test.txt";
    let data = b"test data";
    client.put_object().bucket(bucket).key(key).body(ByteStream::from_static(data)).send().await.unwrap();
    let resp = client.get_object().bucket(bucket).key(key).send().await.unwrap();
    let content = resp.body.collect().await.unwrap().to_vec();
    assert_eq!(content.as_slice(), data);

    // test 4: invalid policy rejected
    let invalid_policy = r#"{"invalid json"#;
    let result = client.put_bucket_policy().bucket(bucket).policy(invalid_policy).send().await;
    assert!(result.is_err());

    // test 5: delete policy
    client.delete_bucket_policy().bucket(bucket).send().await.unwrap();
    let result = client.get_bucket_policy().bucket(bucket).send().await;
    assert!(result.is_err());

    // cleanup
    client.delete_object().bucket(bucket).key(key).send().await.unwrap();
    client.delete_bucket().bucket(bucket).send().await.unwrap();
}
