use super::helpers::{create_aws_client, create_test_server};

#[tokio::test(flavor = "multi_thread")]
async fn policy_advanced_test() {
    let (_addr, endpoint, handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = "policy-adv-bucket";

    client.create_bucket().bucket(bucket).send().await.unwrap();

    // test Deny priority
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
    client.put_bucket_policy().bucket(bucket).policy(deny_policy).send().await.unwrap();

    // test wildcard
    let wildcard_policy = r#"{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": "*",
            "Action": ["s3:*"],
            "Resource": ["arn:aws:s3:::policy-adv-bucket/public/*"]
        }]
    }"#;
    client.put_bucket_policy().bucket(bucket).policy(wildcard_policy).send().await.unwrap();

    let policy = client.get_bucket_policy().bucket(bucket).send().await.unwrap();
    assert!(policy.policy().unwrap().contains("public"));

    client.delete_bucket_policy().bucket(bucket).send().await.unwrap();
    client.delete_bucket().bucket(bucket).send().await.unwrap();
    handle.abort();
}
