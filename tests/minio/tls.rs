use crate::helpers::*;

#[tokio::test]
async fn test_tls_connection() {
    let client = setup_client().await;

    // TODO: implement TLS termination
}
