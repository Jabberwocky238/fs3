use super::helpers::create_minio_server;

#[tokio::test(flavor = "multi_thread")]
async fn content_md5_test() {
    let (_addr, endpoint, handle) = create_minio_server().await.unwrap();
    let client = reqwest::Client::new();
    let bucket = "md5test";
    let key = "test.txt";
    let data = b"hello world";

    // 创建bucket
    client.put(format!("{}/{}", endpoint, bucket)).send().await.unwrap();

    // 正确的MD5
    use base64::{Engine as _, engine::general_purpose};
    let correct_md5 = general_purpose::STANDARD.encode(md5::compute(data).0);
    let resp = client.put(format!("{}/{}/{}", endpoint, bucket, key))
        .header("Content-MD5", &correct_md5)
        .body(data.to_vec())
        .send().await.unwrap();
    assert!(resp.status().is_success());

    // 错误的MD5
    let wrong_md5 = general_purpose::STANDARD.encode(md5::compute(b"wrong").0);
    let resp = client.put(format!("{}/{}/{}", endpoint, bucket, key))
        .header("Content-MD5", &wrong_md5)
        .body(data.to_vec())
        .send().await.unwrap();
    assert!(!resp.status().is_success());

    // 清理
    client.delete(format!("{}/{}/{}", endpoint, bucket, key)).send().await.unwrap();
    client.delete(format!("{}/{}", endpoint, bucket)).send().await.unwrap();
    handle.abort();
}
