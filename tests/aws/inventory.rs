use super::helpers::*;
use aws_sdk_s3::types::{InventoryConfiguration, InventoryDestination, InventoryS3BucketDestination, InventoryFormat, InventoryFrequency, InventoryIncludedObjectVersions, InventorySchedule};

#[tokio::test]
async fn test_put_bucket_inventory() {
    let (_addr, endpoint, _handle) = create_test_server().await.unwrap();
    let client = create_aws_client(&endpoint);
    let bucket = random_bucket_name();
    client.create_bucket().bucket(&bucket).send().await.unwrap();

    let config = InventoryConfiguration::builder()
        .id("inventory1")
        .is_enabled(true)
        .included_object_versions(InventoryIncludedObjectVersions::All)
        .destination(InventoryDestination::builder()
            .s3_bucket_destination(InventoryS3BucketDestination::builder()
                .bucket(format!("arn:aws:s3:::{}", bucket))
                .format(InventoryFormat::Csv)
                .build().unwrap())
            .build())
        .schedule(InventorySchedule::builder().frequency(InventoryFrequency::Daily).build())
        .build().unwrap();

    client.put_bucket_inventory_configuration().bucket(&bucket).id("inventory1").inventory_configuration(config).send().await.unwrap();
}
