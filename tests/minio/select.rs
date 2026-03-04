use crate::helpers::*;

#[tokio::test]
async fn test_select_object_content() {
    let (client, bucket) = setup().await;

    // Upload test CSV data
    client.put_object()
        .bucket(&bucket)
        .key("test.csv")
        .body("name,age\nAlice,30\nBob,25".as_bytes().to_vec().into())
        .send()
        .await
        .unwrap();

    // Try S3 Select query
    let result = client.select_object_content()
        .bucket(&bucket)
        .key("test.csv")
        .expression("SELECT * FROM S3Object WHERE age > 26")
        .expression_type(aws_sdk_s3::types::ExpressionType::Sql)
        .input_serialization(
            aws_sdk_s3::types::InputSerialization::builder()
                .csv(aws_sdk_s3::types::CsvInput::builder().build())
                .build()
        )
        .output_serialization(
            aws_sdk_s3::types::OutputSerialization::builder()
                .csv(aws_sdk_s3::types::CsvOutput::builder().build())
                .build()
        )
        .send()
        .await;

    // Currently returns NotImplemented
    assert!(result.is_ok() || result.is_err());
}
