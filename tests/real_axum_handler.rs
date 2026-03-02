use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::http::StatusCode;
use chrono::Utc;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::types::S3Api;
use minio::s3::{Client, ClientBuilder};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use s3_mount_gateway_rust::axum_router;
use s3_mount_gateway_rust::types::s3::request::*;
use s3_mount_gateway_rust::types::s3::response::*;
use s3_mount_gateway_rust::types::traits::s3_handler::{BucketS3Handler, ObjectS3Handler, RejectedS3Handler, RootS3Handler};

#[derive(Debug, Clone, Default)]
struct RealAxumHandler {
    buckets: Arc<RwLock<HashMap<String, HashMap<String, Vec<u8>>>>>,
}

#[derive(Debug, Clone)]
struct HandlerErr(String);

impl HandlerErr {
    fn unsupported(api: &str) -> Self {
        Self(format!("unsupported api: {api}"))
    }
}

impl std::fmt::Display for HandlerErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for HandlerErr {}

fn ok_meta() -> ResponseMeta {
    ResponseMeta {
        status_code: 200,
        ..ResponseMeta::default()
    }
}

fn object_info(bucket: &str, key: &str, body: &[u8]) -> ObjectInfo {
    ObjectInfo {
        bucket: bucket.to_string(),
        key: key.to_string(),
        size: body.len() as u64,
        etag: Some(format!("{:x}", md5::compute(body))),
        last_modified: Some(Utc::now().to_rfc3339()),
        storage_class: Some("STANDARD".to_string()),
    }
}

fn unsupported<T>(api: &str) -> Result<T, HandlerErr> {
    Err(HandlerErr::unsupported(api))
}

#[async_trait]
impl ObjectS3Handler for RealAxumHandler {
    type Error = HandlerErr;

    async fn head_object(&self, req: HeadObjectRequest) -> Result<HeadObjectResponse, Self::Error> {
        let g = self.buckets.read().await;
        let Some(bucket) = g.get(&req.object.bucket) else {
            return unsupported("head_object");
        };
        let Some(body) = bucket.get(&req.object.object) else {
            return unsupported("head_object");
        };
        Ok(HeadObjectResponse {
            meta: ok_meta(),
            headers: HashMap::new(),
            object: Some(object_info(&req.object.bucket, &req.object.object, body)),
        })
    }

    async fn get_object_attributes(&self, _req: GetObjectAttributesRequest) -> Result<GetObjectAttributesResponse, Self::Error> { unsupported("get_object_attributes") }
    async fn copy_object_part(&self, _req: CopyObjectPartRequest) -> Result<CopyObjectPartResponse, Self::Error> { unsupported("copy_object_part") }
    async fn put_object_part(&self, _req: PutObjectPartRequest) -> Result<PutObjectPartResponse, Self::Error> { unsupported("put_object_part") }
    async fn list_object_parts(&self, _req: ListObjectPartsRequest) -> Result<ListObjectPartsResponse, Self::Error> { unsupported("list_object_parts") }
    async fn complete_multipart_upload(&self, _req: CompleteMultipartUploadRequest) -> Result<CompleteMultipartUploadResponse, Self::Error> { unsupported("complete_multipart_upload") }
    async fn new_multipart_upload(&self, _req: NewMultipartUploadRequest) -> Result<NewMultipartUploadResponse, Self::Error> { unsupported("new_multipart_upload") }
    async fn abort_multipart_upload(&self, _req: AbortMultipartUploadRequest) -> Result<AbortMultipartUploadResponse, Self::Error> { unsupported("abort_multipart_upload") }
    async fn get_object_acl(&self, _req: GetObjectAclRequest) -> Result<GetObjectAclResponse, Self::Error> { unsupported("get_object_acl") }
    async fn put_object_acl(&self, _req: PutObjectAclRequest) -> Result<PutObjectAclResponse, Self::Error> { unsupported("put_object_acl") }
    async fn get_object_tagging(&self, _req: GetObjectTaggingRequest) -> Result<GetObjectTaggingResponse, Self::Error> { unsupported("get_object_tagging") }
    async fn put_object_tagging(&self, _req: PutObjectTaggingRequest) -> Result<PutObjectTaggingResponse, Self::Error> { unsupported("put_object_tagging") }
    async fn delete_object_tagging(&self, _req: DeleteObjectTaggingRequest) -> Result<DeleteObjectTaggingResponse, Self::Error> { unsupported("delete_object_tagging") }
    async fn select_object_content(&self, _req: SelectObjectContentRequest) -> Result<SelectObjectContentResponse, Self::Error> { unsupported("select_object_content") }
    async fn get_object_retention(&self, _req: GetObjectRetentionRequest) -> Result<GetObjectRetentionResponse, Self::Error> { unsupported("get_object_retention") }
    async fn get_object_legal_hold(&self, _req: GetObjectLegalHoldRequest) -> Result<GetObjectLegalHoldResponse, Self::Error> { unsupported("get_object_legal_hold") }
    async fn get_object_lambda(&self, _req: GetObjectLambdaRequest) -> Result<GetObjectLambdaResponse, Self::Error> { unsupported("get_object_lambda") }

    async fn get_object(&self, req: GetObjectRequest) -> Result<GetObjectResponse, Self::Error> {
        let g = self.buckets.read().await;
        let Some(bucket) = g.get(&req.object.bucket) else {
            return unsupported("get_object");
        };
        let Some(body) = bucket.get(&req.object.object) else {
            return unsupported("get_object");
        };
        Ok(GetObjectResponse {
            meta: ok_meta(),
            body: body.clone(),
        })
    }

    async fn copy_object(&self, _req: CopyObjectRequest) -> Result<CopyObjectResponse, Self::Error> { unsupported("copy_object") }
    async fn put_object_retention(&self, _req: PutObjectRetentionRequest) -> Result<PutObjectRetentionResponse, Self::Error> { unsupported("put_object_retention") }
    async fn put_object_legal_hold(&self, _req: PutObjectLegalHoldRequest) -> Result<PutObjectLegalHoldResponse, Self::Error> { unsupported("put_object_legal_hold") }
    async fn put_object_extract(&self, _req: PutObjectExtractRequest) -> Result<PutObjectExtractResponse, Self::Error> { unsupported("put_object_extract") }
    async fn append_object_rejected(&self, _req: AppendObjectRejectedRequest) -> Result<AppendObjectRejectedResponse, Self::Error> { unsupported("append_object_rejected") }

    async fn put_object(&self, req: PutObjectRequest) -> Result<PutObjectResponse, Self::Error> {
        let mut g = self.buckets.write().await;
        let bucket = g.entry(req.object.bucket.clone()).or_default();
        bucket.insert(req.object.object.clone(), req.body.clone());
        Ok(PutObjectResponse {
            meta: ok_meta(),
            object: Some(object_info(&req.object.bucket, &req.object.object, &req.body)),
        })
    }

    async fn delete_object(&self, req: DeleteObjectRequest) -> Result<DeleteObjectResponse, Self::Error> {
        let mut g = self.buckets.write().await;
        if let Some(bucket) = g.get_mut(&req.object.bucket) {
            bucket.remove(&req.object.object);
        }
        Ok(DeleteObjectResponse { meta: ok_meta() })
    }

    async fn post_restore_object(&self, _req: PostRestoreObjectRequest) -> Result<PostRestoreObjectResponse, Self::Error> { unsupported("post_restore_object") }
}

#[async_trait]
impl BucketS3Handler for RealAxumHandler {
    type Error = HandlerErr;

    async fn get_bucket_location(&self, _req: GetBucketLocationRequest) -> Result<GetBucketLocationResponse, Self::Error> { unsupported("get_bucket_location") }
    async fn get_bucket_policy(&self, _req: GetBucketPolicyRequest) -> Result<GetBucketPolicyResponse, Self::Error> { unsupported("get_bucket_policy") }
    async fn get_bucket_lifecycle(&self, _req: GetBucketLifecycleRequest) -> Result<GetBucketLifecycleResponse, Self::Error> { unsupported("get_bucket_lifecycle") }
    async fn get_bucket_encryption(&self, _req: GetBucketEncryptionRequest) -> Result<GetBucketEncryptionResponse, Self::Error> { unsupported("get_bucket_encryption") }
    async fn get_bucket_object_lock_config(&self, _req: GetBucketObjectLockConfigRequest) -> Result<GetBucketObjectLockConfigResponse, Self::Error> { unsupported("get_bucket_object_lock_config") }
    async fn get_bucket_replication_config(&self, _req: GetBucketReplicationConfigRequest) -> Result<GetBucketReplicationConfigResponse, Self::Error> { unsupported("get_bucket_replication_config") }
    async fn get_bucket_versioning(&self, _req: GetBucketVersioningRequest) -> Result<GetBucketVersioningResponse, Self::Error> { unsupported("get_bucket_versioning") }
    async fn get_bucket_notification(&self, _req: GetBucketNotificationRequest) -> Result<GetBucketNotificationResponse, Self::Error> { unsupported("get_bucket_notification") }
    async fn listen_bucket_notification(&self, _req: ListenBucketNotificationRequest) -> Result<ListenBucketNotificationResponse, Self::Error> { unsupported("listen_bucket_notification") }
    async fn reset_bucket_replication_status(&self, _req: ResetBucketReplicationStatusRequest) -> Result<ResetBucketReplicationStatusResponse, Self::Error> { unsupported("reset_bucket_replication_status") }
    async fn get_bucket_acl(&self, _req: GetBucketAclRequest) -> Result<GetBucketAclResponse, Self::Error> { unsupported("get_bucket_acl") }
    async fn put_bucket_acl(&self, _req: PutBucketAclRequest) -> Result<PutBucketAclResponse, Self::Error> { unsupported("put_bucket_acl") }
    async fn get_bucket_cors(&self, _req: GetBucketCorsRequest) -> Result<GetBucketCorsResponse, Self::Error> { unsupported("get_bucket_cors") }
    async fn put_bucket_cors(&self, _req: PutBucketCorsRequest) -> Result<PutBucketCorsResponse, Self::Error> { unsupported("put_bucket_cors") }
    async fn delete_bucket_cors(&self, _req: DeleteBucketCorsRequest) -> Result<DeleteBucketCorsResponse, Self::Error> { unsupported("delete_bucket_cors") }
    async fn get_bucket_website(&self, _req: GetBucketWebsiteRequest) -> Result<GetBucketWebsiteResponse, Self::Error> { unsupported("get_bucket_website") }
    async fn get_bucket_accelerate(&self, _req: GetBucketAccelerateRequest) -> Result<GetBucketAccelerateResponse, Self::Error> { unsupported("get_bucket_accelerate") }
    async fn get_bucket_request_payment(&self, _req: GetBucketRequestPaymentRequest) -> Result<GetBucketRequestPaymentResponse, Self::Error> { unsupported("get_bucket_request_payment") }
    async fn get_bucket_logging(&self, _req: GetBucketLoggingRequest) -> Result<GetBucketLoggingResponse, Self::Error> { unsupported("get_bucket_logging") }
    async fn get_bucket_tagging(&self, _req: GetBucketTaggingRequest) -> Result<GetBucketTaggingResponse, Self::Error> { unsupported("get_bucket_tagging") }
    async fn delete_bucket_website(&self, _req: DeleteBucketWebsiteRequest) -> Result<DeleteBucketWebsiteResponse, Self::Error> { unsupported("delete_bucket_website") }
    async fn delete_bucket_tagging(&self, _req: DeleteBucketTaggingRequest) -> Result<DeleteBucketTaggingResponse, Self::Error> { unsupported("delete_bucket_tagging") }
    async fn list_multipart_uploads(&self, _req: ListMultipartUploadsRequest) -> Result<ListMultipartUploadsResponse, Self::Error> { unsupported("list_multipart_uploads") }
    async fn list_objects_v2m(&self, _req: ListObjectsV2MRequest) -> Result<ListObjectsV2MResponse, Self::Error> { unsupported("list_objects_v2m") }

    async fn list_objects_v2(&self, req: ListObjectsV2Request) -> Result<ListObjectsV2Response, Self::Error> {
        let g = self.buckets.read().await;
        let Some(bucket) = g.get(&req.bucket.bucket) else {
            return Ok(ListObjectsV2Response { meta: ok_meta(), objects: vec![] });
        };
        let objects = bucket
            .iter()
            .map(|(key, body)| object_info(&req.bucket.bucket, key, body))
            .collect();
        Ok(ListObjectsV2Response { meta: ok_meta(), objects })
    }

    async fn list_object_versions_m(&self, _req: ListObjectVersionsMRequest) -> Result<ListObjectVersionsMResponse, Self::Error> { unsupported("list_object_versions_m") }
    async fn list_object_versions(&self, _req: ListObjectVersionsRequest) -> Result<ListObjectVersionsResponse, Self::Error> { unsupported("list_object_versions") }
    async fn get_bucket_policy_status(&self, _req: GetBucketPolicyStatusRequest) -> Result<GetBucketPolicyStatusResponse, Self::Error> { unsupported("get_bucket_policy_status") }
    async fn put_bucket_lifecycle(&self, _req: PutBucketLifecycleRequest) -> Result<PutBucketLifecycleResponse, Self::Error> { unsupported("put_bucket_lifecycle") }
    async fn put_bucket_replication_config(&self, _req: PutBucketReplicationConfigRequest) -> Result<PutBucketReplicationConfigResponse, Self::Error> { unsupported("put_bucket_replication_config") }
    async fn put_bucket_encryption(&self, _req: PutBucketEncryptionRequest) -> Result<PutBucketEncryptionResponse, Self::Error> { unsupported("put_bucket_encryption") }
    async fn put_bucket_policy(&self, _req: PutBucketPolicyRequest) -> Result<PutBucketPolicyResponse, Self::Error> { unsupported("put_bucket_policy") }
    async fn put_bucket_object_lock_config(&self, _req: PutBucketObjectLockConfigRequest) -> Result<PutBucketObjectLockConfigResponse, Self::Error> { unsupported("put_bucket_object_lock_config") }
    async fn put_bucket_tagging(&self, _req: PutBucketTaggingRequest) -> Result<PutBucketTaggingResponse, Self::Error> { unsupported("put_bucket_tagging") }
    async fn put_bucket_versioning(&self, _req: PutBucketVersioningRequest) -> Result<PutBucketVersioningResponse, Self::Error> { unsupported("put_bucket_versioning") }
    async fn put_bucket_notification(&self, _req: PutBucketNotificationRequest) -> Result<PutBucketNotificationResponse, Self::Error> { unsupported("put_bucket_notification") }
    async fn reset_bucket_replication_start(&self, _req: ResetBucketReplicationStartRequest) -> Result<ResetBucketReplicationStartResponse, Self::Error> { unsupported("reset_bucket_replication_start") }

    async fn put_bucket(&self, req: PutBucketRequest) -> Result<PutBucketResponse, Self::Error> {
        let mut g = self.buckets.write().await;
        g.entry(req.bucket.bucket).or_default();
        Ok(PutBucketResponse { meta: ok_meta() })
    }

    async fn head_bucket(&self, req: HeadBucketRequest) -> Result<HeadBucketResponse, Self::Error> {
        let g = self.buckets.read().await;
        if g.contains_key(&req.bucket.bucket) {
            Ok(HeadBucketResponse { meta: ok_meta() })
        } else {
            unsupported("head_bucket")
        }
    }

    async fn post_policy(&self, _req: PostPolicyRequest) -> Result<PostPolicyResponse, Self::Error> { unsupported("post_policy") }
    async fn delete_multiple_objects(&self, _req: DeleteMultipleObjectsRequest) -> Result<DeleteMultipleObjectsResponse, Self::Error> { unsupported("delete_multiple_objects") }
    async fn delete_bucket_policy(&self, _req: DeleteBucketPolicyRequest) -> Result<DeleteBucketPolicyResponse, Self::Error> { unsupported("delete_bucket_policy") }
    async fn delete_bucket_replication(&self, _req: DeleteBucketReplicationRequest) -> Result<DeleteBucketReplicationResponse, Self::Error> { unsupported("delete_bucket_replication") }
    async fn delete_bucket_lifecycle(&self, _req: DeleteBucketLifecycleRequest) -> Result<DeleteBucketLifecycleResponse, Self::Error> { unsupported("delete_bucket_lifecycle") }
    async fn delete_bucket_encryption(&self, _req: DeleteBucketEncryptionRequest) -> Result<DeleteBucketEncryptionResponse, Self::Error> { unsupported("delete_bucket_encryption") }

    async fn delete_bucket(&self, req: DeleteBucketRequest) -> Result<DeleteBucketResponse, Self::Error> {
        let mut g = self.buckets.write().await;
        g.remove(&req.bucket.bucket);
        Ok(DeleteBucketResponse { meta: ok_meta() })
    }

    async fn get_bucket_replication_metrics_v2(&self, _req: GetBucketReplicationMetricsV2Request) -> Result<GetBucketReplicationMetricsV2Response, Self::Error> { unsupported("get_bucket_replication_metrics_v2") }
    async fn get_bucket_replication_metrics(&self, _req: GetBucketReplicationMetricsRequest) -> Result<GetBucketReplicationMetricsResponse, Self::Error> { unsupported("get_bucket_replication_metrics") }
    async fn validate_bucket_replication_creds(&self, _req: ValidateBucketReplicationCredsRequest) -> Result<ValidateBucketReplicationCredsResponse, Self::Error> { unsupported("validate_bucket_replication_creds") }

    async fn list_objects_v1(&self, req: ListObjectsV1Request) -> Result<ListObjectsV1Response, Self::Error> {
        let g = self.buckets.read().await;
        let Some(bucket) = g.get(&req.bucket.bucket) else {
            return Ok(ListObjectsV1Response { meta: ok_meta(), objects: vec![] });
        };
        let objects = bucket
            .iter()
            .map(|(key, body)| object_info(&req.bucket.bucket, key, body))
            .collect();
        Ok(ListObjectsV1Response { meta: ok_meta(), objects })
    }
}

#[async_trait]
impl RootS3Handler for RealAxumHandler {
    type Error = HandlerErr;

    async fn root_listen_notification(&self, _req: RootListenNotificationRequest) -> Result<RootListenNotificationResponse, Self::Error> {
        unsupported("root_listen_notification")
    }

    async fn list_buckets(&self, _req: ListBucketsRequest) -> Result<ListBucketsResponse, Self::Error> {
        let g = self.buckets.read().await;
        let buckets = g
            .keys()
            .map(|name| BucketInfo {
                name: name.clone(),
                creation_date: Some(Utc::now().to_rfc3339()),
            })
            .collect();
        Ok(ListBucketsResponse {
            meta: ok_meta(),
            buckets,
        })
    }

    async fn list_buckets_double_slash(&self, _req: ListBucketsDoubleSlashRequest) -> Result<ListBucketsDoubleSlashResponse, Self::Error> {
        let g = self.buckets.read().await;
        let buckets = g
            .keys()
            .map(|name| BucketInfo {
                name: name.clone(),
                creation_date: Some(Utc::now().to_rfc3339()),
            })
            .collect();
        Ok(ListBucketsDoubleSlashResponse {
            meta: ok_meta(),
            buckets,
        })
    }
}

#[async_trait]
impl RejectedS3Handler for RealAxumHandler {
    type Error = HandlerErr;

    async fn rejected_object_torrent(&self, _req: RejectedObjectTorrentRequest) -> Result<RejectedApiResponse, Self::Error> {
        Ok(RejectedApiResponse {
            meta: ResponseMeta {
                status_code: 400,
                ..ResponseMeta::default()
            },
            error: ErrorBody {
                code: "Rejected".to_string(),
                message: "rejected api".to_string(),
                resource: None,
            },
        })
    }

    async fn rejected_object_acl_delete(&self, _req: RejectedObjectAclDeleteRequest) -> Result<RejectedApiResponse, Self::Error> {
        self.rejected_object_torrent(RejectedObjectTorrentRequest::default())
            .await
    }

    async fn rejected_bucket_api(&self, _req: RejectedBucketApiRequest) -> Result<RejectedApiResponse, Self::Error> {
        self.rejected_object_torrent(RejectedObjectTorrentRequest::default())
            .await
    }
}

async fn start_server() -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind real port failed");
    let addr = listener.local_addr().expect("local addr failed");
    let base = format!("http://{}", addr);

    let app = axum_router::<RealAxumHandler, HandlerErr>(RealAxumHandler::default());
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let client = reqwest::Client::new();
    for _ in 0..100 {
        if client.get(format!("{base}/")).send().await.is_ok() {
            return (base, handle);
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("server not ready")
}

fn minio_client(base: &str, access_key: &str, secret_key: &str) -> Client {
    let base_url = base.parse::<BaseUrl>().expect("invalid base url");
    let provider = StaticProvider::new(access_key, secret_key, None);
    ClientBuilder::new(base_url)
        .provider(Some(Box::new(provider)))
        .build()
        .expect("build minio client failed")
}

#[tokio::test(flavor = "multi_thread")]
async fn real_axum_handler_real_start_real_port() {
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();
    let minio = minio_client(&base, "ak-test", "sk-test");
    let payload_text = "hello-real-axum-from-minio";

    minio
        .create_bucket("docs")
        .send()
        .await
        .expect("minio create_bucket failed");
    minio
        .put_object_content("docs", "hello.txt", payload_text.to_string())
        .send()
        .await
        .expect("minio put_object_content failed");

    let get_object = client
        .get(format!("{base}/docs/hello.txt"))
        .send()
        .await
        .expect("get object failed");
    assert_eq!(get_object.status(), StatusCode::OK);
    let payload: serde_json::Value = get_object
        .json()
        .await
        .expect("parse get json failed");
    assert_eq!(payload["api"], "GetObject");
    assert_eq!(
        payload["response"]["body"],
        serde_json::json!(payload_text.as_bytes())
    );

    let list_buckets = client
        .get(format!("{base}/"))
        .send()
        .await
        .expect("list buckets failed");
    assert_eq!(list_buckets.status(), StatusCode::OK);
    let root: serde_json::Value = list_buckets
        .json()
        .await
        .expect("parse list buckets json failed");
    assert_eq!(root["api"], "ListBuckets");
    assert_eq!(root["response"]["buckets"][0]["name"], "docs");

    handle.abort();
}
