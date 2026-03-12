use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Method};
use axum::routing::any;
use axum::Router;
use futures::StreamExt as _;

use crate::types::s3::request::*;
use crate::types::s3::response::S3Response;
use crate::types::s3::xml;
use crate::types::errors::S3EngineError;
use crate::types::traits::s3_handler::{S3Handler, S3HandlerBridgeError};

use super::util::{get, has, header, list_query};
use super::HandlerError;
use super::BucketError;

fn bucket_err(e: impl std::fmt::Display) -> HandlerError {
    let msg = e.to_string();
    HandlerError::Bucket(if msg.contains("bucket not found") {
        BucketError::NotFound(msg)
    } else if msg.contains("already exists") {
        BucketError::AlreadyExists(msg)
    } else if msg.contains("not empty") {
        BucketError::NotEmpty(msg)
    } else {
        BucketError::Internal(msg)
    })
}

pub fn routes<T, E>(state: Arc<T>) -> Router
where
    T: S3Handler<E> + Send + Sync + 'static,
    E: std::fmt::Display + From<S3HandlerBridgeError> + From<S3EngineError> + 'static,
{
    Router::new()
        .route("/{bucket}", any(bucket_entry::<T, E>))
        .route("/{bucket}/", any(bucket_entry::<T, E>))
        .with_state(state)
}

fn body_stream(body: Body) -> crate::types::s3::core::BoxByteStream {
    Box::pin(body.into_data_stream().map(|result| {
        result.map_err(|err| io::Error::other(err.to_string()))
    }))
}

async fn body_text(body: Body) -> Result<String, S3HandlerBridgeError> {
    let stream = body_stream(body);
    crate::types::traits::s3_handler::utils::stream_to_string(stream).await
}

async fn bucket_entry<T, E>(
    State(handler): State<Arc<T>>,
    Path(bucket_name): Path<String>,
    method: Method,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    body: Body,
) -> Result<S3Response, HandlerError>
where
    T: S3Handler<E> + Send + Sync,
    E: std::fmt::Display + From<S3HandlerBridgeError> + From<S3EngineError> + 'static,
{
    let mk = || BucketRef { bucket: bucket_name.clone() };
    if let Some(api) = rejected_api(&q, &method) {
        let v = handler
            .rejected_bucket_api(RejectedBucketApiRequest {
                bucket: mk(),
                api: api.to_string(),
                method: method.to_string(),
            })
            .await
            .map_err(bucket_err)?;
        return Ok(S3Response::RejectedApi(v));
    }
    let resp = match method {
        Method::GET if has(&q, "location") => S3Response::GetBucketLocation(handler.get_bucket_location(GetBucketLocationRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "policy") => S3Response::GetBucketPolicy(handler.get_bucket_policy(GetBucketPolicyRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "lifecycle") => S3Response::GetBucketLifecycle(handler.get_bucket_lifecycle(GetBucketLifecycleRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "encryption") => S3Response::GetBucketEncryption(handler.get_bucket_encryption(GetBucketEncryptionRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "object-lock") => S3Response::GetBucketObjectLockConfig(handler.get_bucket_object_lock_config(GetBucketObjectLockConfigRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "replication") => S3Response::GetBucketReplicationConfig(handler.get_bucket_replication_config(GetBucketReplicationConfigRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "versioning") => S3Response::GetBucketVersioning(handler.get_bucket_versioning(GetBucketVersioningRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "notification") => S3Response::GetBucketNotification(handler.get_bucket_notification(GetBucketNotificationRequest { bucket: mk() }).await.map_err(bucket_err)?),
        // Method::GET if has(&q, "events") => S3Response::ListenBucketNotification(handler.listen_bucket_notification(ListenBucketNotificationRequest { bucket: mk(), filter: event_filter(&q) }).await.map_err(bucket_err)?),
        // Method::GET if has(&q, "replication-reset-status") => S3Response::ResetBucketReplicationStatus(handler.reset_bucket_replication_status(ResetBucketReplicationStatusRequest { bucket: mk() }).await.map_err(bucket_err)?),
        // Method::GET if has(&q, "acl") => S3Response::GetBucketAcl(handler.get_bucket_acl(GetBucketAclRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "cors") => S3Response::GetBucketCors(handler.get_bucket_cors(GetBucketCorsRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "website") => S3Response::GetBucketWebsite(handler.get_bucket_website(GetBucketWebsiteRequest { bucket: mk() }).await.map_err(bucket_err)?),
        // Method::GET if has(&q, "accelerate") => S3Response::GetBucketAccelerate(handler.get_bucket_accelerate(GetBucketAccelerateRequest { bucket: mk() }).await.map_err(bucket_err)?),
        // Method::GET if has(&q, "requestPayment") => S3Response::GetBucketRequestPayment(handler.get_bucket_request_payment(GetBucketRequestPaymentRequest { bucket: mk() }).await.map_err(bucket_err)?),
        // Method::GET if has(&q, "logging") => S3Response::GetBucketLogging(handler.get_bucket_logging(GetBucketLoggingRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "tagging") => S3Response::GetBucketTagging(handler.get_bucket_tagging(GetBucketTaggingRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "uploads") => S3Response::ListMultipartUploads(handler.list_multipart_uploads(ListMultipartUploadsRequest { bucket: mk(), query: list_query(&q) }).await.map_err(bucket_err)?),
        Method::GET if get(&q, "list-type").as_deref() == Some("2") && get(&q, "metadata").as_deref() == Some("true") => S3Response::ListObjectsV2M(handler.list_objects_v2m(ListObjectsV2MRequest { bucket: mk(), query: list_query(&q), metadata: true }).await.map_err(bucket_err)?),
        Method::GET if get(&q, "list-type").as_deref() == Some("2") => S3Response::ListObjectsV2(handler.list_objects_v2(ListObjectsV2Request { bucket: mk(), query: list_query(&q) }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "versions") && get(&q, "metadata").as_deref() == Some("true") => S3Response::ListObjectVersionsM(handler.list_object_versions_m(ListObjectVersionsMRequest { bucket: mk(), query: list_query(&q), metadata: true }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "versions") => S3Response::ListObjectVersions(handler.list_object_versions(ListObjectVersionsRequest { bucket: mk(), query: list_query(&q) }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "policyStatus") => S3Response::GetBucketPolicyStatus(handler.get_bucket_policy_status(GetBucketPolicyStatusRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if get(&q, "replication-metrics").as_deref() == Some("2") => S3Response::GetBucketReplicationMetricsV2(handler.get_bucket_replication_metrics_v2(GetBucketReplicationMetricsV2Request { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "replication-metrics") => S3Response::GetBucketReplicationMetrics(handler.get_bucket_replication_metrics(GetBucketReplicationMetricsRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET if has(&q, "replication-check") => S3Response::ValidateBucketReplicationCreds(handler.validate_bucket_replication_creds(ValidateBucketReplicationCredsRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::GET => S3Response::ListObjectsV1(handler.list_objects_v1(ListObjectsV1Request { bucket: mk(), query: list_query(&q) }).await.map_err(bucket_err)?),

        Method::PUT if has(&q, "lifecycle") => {
            let xml = body_text(body).await.map_err(bucket_err)?;
            let rules = xml::parse_lifecycle(&xml).map_err(bucket_err)?;
            S3Response::PutBucketLifecycle(handler.put_bucket_lifecycle(PutBucketLifecycleRequest { bucket: mk(), rules }).await.map_err(bucket_err)?)
        }
        Method::PUT if has(&q, "replication") => {
            let xml = body_text(body).await.map_err(bucket_err)?;
            let replication = xml::parse_replication(&xml).map_err(bucket_err)?;
            S3Response::PutBucketReplicationConfig(handler.put_bucket_replication_config(PutBucketReplicationConfigRequest { bucket: mk(), replication }).await.map_err(bucket_err)?)
        }
        Method::PUT if has(&q, "encryption") => {
            let xml = body_text(body).await.map_err(bucket_err)?;
            let encryption = xml::parse_encryption(&xml).map_err(bucket_err)?;
            S3Response::PutBucketEncryption(handler.put_bucket_encryption(PutBucketEncryptionRequest { bucket: mk(), encryption }).await.map_err(bucket_err)?)
        }
        Method::PUT if has(&q, "policy") => {
            let json = body_text(body).await.map_err(bucket_err)?;
            S3Response::PutBucketPolicy(handler.put_bucket_policy(PutBucketPolicyRequest { bucket: mk(), json }).await.map_err(bucket_err)?)
        }
        Method::PUT if has(&q, "object-lock") => {
            let xml = body_text(body).await.map_err(bucket_err)?;
            let config = xml::parse_object_lock(&xml).map_err(bucket_err)?;
            S3Response::PutBucketObjectLockConfig(handler.put_bucket_object_lock_config(PutBucketObjectLockConfigRequest { bucket: mk(), config }).await.map_err(bucket_err)?)
        }
        Method::PUT if has(&q, "tagging") => {
            let xml = body_text(body).await.map_err(bucket_err)?;
            let tags = xml::parse_tagging(&xml).map_err(bucket_err)?;
            S3Response::PutBucketTagging(handler.put_bucket_tagging(PutBucketTaggingRequest { bucket: mk(), tags }).await.map_err(bucket_err)?)
        }
        Method::PUT if has(&q, "versioning") => {
            let xml = body_text(body).await.map_err(bucket_err)?;
            let versioning = xml::parse_versioning(&xml).map_err(bucket_err)?;
            S3Response::PutBucketVersioning(handler.put_bucket_versioning(PutBucketVersioningRequest { bucket: mk(), versioning }).await.map_err(bucket_err)?)
        }
        Method::PUT if has(&q, "notification") => {
            let xml = body_text(body).await.map_err(bucket_err)?;
            let configs = xml::parse_notification(&xml).map_err(bucket_err)?;
            S3Response::PutBucketNotification(handler.put_bucket_notification(PutBucketNotificationRequest { bucket: mk(), configs }).await.map_err(bucket_err)?)
        }
        Method::PUT if has(&q, "website") => {
            let xml = body_text(body).await.map_err(bucket_err)?;
            let website = xml::parse_website(&xml).map_err(bucket_err)?;
            S3Response::PutBucketWebsite(handler.put_bucket_website(PutBucketWebsiteRequest { bucket: mk(), website }).await.map_err(bucket_err)?)
        }
        // Method::PUT if has(&q, "replication-reset") => S3Response::ResetBucketReplicationStart(handler.reset_bucket_replication_start(ResetBucketReplicationStartRequest { bucket: mk() }).await.map_err(bucket_err)?),
        // Method::PUT if has(&q, "acl") => S3Response::PutBucketAcl(handler.put_bucket_acl(PutBucketAclRequest { bucket: mk(), xml: body_string(&body) }).await.map_err(bucket_err)?),
        Method::PUT if has(&q, "cors") => {
            let xml = body_text(body).await.map_err(bucket_err)?;
            let cors = xml::parse_cors(&xml).map_err(bucket_err)?;
            S3Response::PutBucketCors(handler.put_bucket_cors(PutBucketCorsRequest { bucket: mk(), cors }).await.map_err(bucket_err)?)
        }
        Method::PUT => S3Response::PutBucket(handler.put_bucket(PutBucketRequest { bucket: mk(), region: header(&headers, "x-amz-bucket-region") }).await.map_err(bucket_err)?),

        Method::POST if has(&q, "delete") => {
            let xml = body_text(body).await.map_err(bucket_err)?;
            let payload = xml::parse_delete_objects(&xml).map_err(bucket_err)?;
            S3Response::DeleteMultipleObjects(handler.delete_multiple_objects(DeleteMultipleObjectsRequest {
                bucket: mk(),
                payload: DeleteObjectsInput { quiet: payload.quiet, keys: payload.keys },
            }).await.map_err(bucket_err)?)
        }
        // Method::POST => S3Response::PostPolicy(handler.post_policy(PostPolicyRequest { bucket: mk(), form: PostPolicyForm { fields: HashMap::new(), payload: body_stream() } }).await.map_err(bucket_err)?),

        Method::DELETE if has(&q, "policy") => S3Response::DeleteBucketPolicy(handler.delete_bucket_policy(DeleteBucketPolicyRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::DELETE if has(&q, "replication") => S3Response::DeleteBucketReplication(handler.delete_bucket_replication(DeleteBucketReplicationRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::DELETE if has(&q, "lifecycle") => S3Response::DeleteBucketLifecycle(handler.delete_bucket_lifecycle(DeleteBucketLifecycleRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::DELETE if has(&q, "encryption") => S3Response::DeleteBucketEncryption(handler.delete_bucket_encryption(DeleteBucketEncryptionRequest { bucket: mk() }).await.map_err(bucket_err)?),
        // Method::DELETE if has(&q, "website") => S3Response::DeleteBucketWebsite(handler.delete_bucket_website(DeleteBucketWebsiteRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::DELETE if has(&q, "tagging") => S3Response::DeleteBucketTagging(handler.delete_bucket_tagging(DeleteBucketTaggingRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::DELETE if has(&q, "cors") => S3Response::DeleteBucketCors(handler.delete_bucket_cors(DeleteBucketCorsRequest { bucket: mk() }).await.map_err(bucket_err)?),
        Method::DELETE => S3Response::DeleteBucket(handler.delete_bucket(DeleteBucketRequest { bucket: mk() }).await.map_err(bucket_err)?),

        Method::HEAD => S3Response::HeadBucket(handler.head_bucket(HeadBucketRequest { bucket: mk() }).await.map_err(bucket_err)?),
        _ => return Err(HandlerError::method_not_allowed("unsupported bucket API")),
    };
    Ok(resp)
}

fn rejected_api(q: &HashMap<String, String>, method: &Method) -> Option<&'static str> {
    let m = method.clone();
    if has(q, "inventory") && matches!(m, Method::GET | Method::PUT | Method::DELETE) { return Some("inventory"); }
    if has(q, "metrics") && matches!(m, Method::GET | Method::PUT | Method::DELETE) { return Some("metrics"); }
    if has(q, "publicAccessBlock") && matches!(m, Method::GET | Method::PUT | Method::DELETE) { return Some("publicAccessBlock"); }
    if has(q, "ownershipControls") && matches!(m, Method::GET | Method::PUT | Method::DELETE) { return Some("ownershipControls"); }
    if has(q, "intelligent-tiering") && matches!(m, Method::GET | Method::PUT | Method::DELETE) { return Some("intelligent-tiering"); }
    if has(q, "analytics") && matches!(m, Method::GET | Method::PUT | Method::DELETE) { return Some("analytics"); }
    if has(q, "logging") && matches!(m, Method::PUT | Method::DELETE) { return Some("logging"); }
    if has(q, "accelerate") && matches!(m, Method::PUT | Method::DELETE) { return Some("accelerate"); }
    if has(q, "requestPayment") && matches!(m, Method::PUT | Method::DELETE) { return Some("requestPayment"); }
    if has(q, "acl") && matches!(m, Method::HEAD | Method::DELETE) { return Some("acl"); }
    None
}
