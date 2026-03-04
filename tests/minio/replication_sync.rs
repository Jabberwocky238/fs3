use crate::helpers::*;
use aws_sdk_s3::types::{ReplicationConfiguration, ReplicationRule, Destination, ReplicationRuleStatus};

#[tokio::test]
async fn test_replication_sync() {
    let client = setup_client().await;
    let bucket = random_bucket_name();
    let dest_bucket = random_bucket_name();
    client.create_bucket(&bucket).send().await.unwrap();
    client.create_bucket(&dest_bucket).send().await.unwrap();

    let replication = ReplicationConfiguration::builder()
        .role("arn:aws:iam::123456789012:role/replication")
        .rules(ReplicationRule::builder()
            .id("rule1")
            .status(ReplicationRuleStatus::Enabled)
            .destination(Destination::builder()
                .bucket(format!("arn:aws:s3:::{}", dest_bucket))
                .build().unwrap())
            .build().unwrap())
        .build().unwrap();

    client.put_bucket_replication().bucket(&bucket).replication_configuration(replication).send().await.unwrap();
}
