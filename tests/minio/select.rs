use minio::s3::builders::ObjectContent;
use minio::s3::types::{S3Api, CsvInputSerialization, CsvOutputSerialization, SelectRequest};

use super::helpers::{create_minio_client, create_minio_server};

#[tokio::test(flavor = "multi_thread")]
async fn test_select_object_content() {
    let (_addr, endpoint, _task) = create_minio_server().await.unwrap();
    let client = create_minio_client(&endpoint).unwrap();
    let bucket = "select-test";
    let key = "data.csv";

    // Create bucket
    client.create_bucket(bucket).send().await.unwrap();

    // Upload CSV data
    let csv_data = b"name,age\nAlice,30\nBob,25\nCharlie,35";
    client
        .put_object_content(bucket, key, ObjectContent::from(csv_data.as_ref()))
        .send()
        .await
        .unwrap();

    // Execute S3 Select query
    let request = SelectRequest::new_csv_input_output(
        "SELECT * FROM S3Object WHERE CAST(age AS INT) > 26",
        CsvInputSerialization::default(),
        CsvOutputSerialization::default(),
    )
    .unwrap();

    let result = client
        .select_object_content(bucket, key, request)
        .send()
        .await;

    // Should return unsupported error
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = format!("{:?}", err);
    assert!(err_str.contains("unsupported") || err_str.contains("SelectObjectContent"));
}
