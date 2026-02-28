#![cfg(all(feature = "policy", feature = "multi-user"))]
use minio::s3::types::S3Api;
mod helpers;

#[tokio::test(flavor = "multi_thread")]
async fn signature_auth_check() {
    let (base, handle) = helpers::start_test_server("signature").await;

    let bad = helpers::minio_client(&base, "wrong-ak", "wrong-sk");
    let bad_put = bad
        .put_object_content("docs", "auth/bad.txt", "x".to_string())
        .send()
        .await
        .err();
    assert!(bad_put.is_some(), "bad credentials should fail");

    let good = helpers::minio_client(&base, "alice-ak", "alice-sk");
    good.put_object_content("docs", "auth/good.txt", "signed-ok".to_string())
        .send()
        .await
        .expect("put with valid signature failed");

    let got = good
        .get_object("docs", "auth/good.txt")
        .send()
        .await
        .expect("get with valid signature failed");
    let body = got
        .content
        .to_segmented_bytes()
        .await
        .expect("read object body failed")
        .to_bytes();
    assert_eq!(body.as_ref(), b"signed-ok");

    handle.abort();
}
