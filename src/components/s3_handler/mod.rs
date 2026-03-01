pub mod bucket;
pub mod object;
pub mod reject;
pub mod root;

use std::fmt::{Display, Formatter};
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};

use crate::types::s3::request::S3Request;
use crate::types::s3::response::S3Response;
use crate::types::traits::s3_handler::{
    BucketS3Handler, ObjectS3Handler, RejectedS3Handler, RootS3Handler,
};

pub trait S3Handler:
    ObjectS3Handler + BucketS3Handler + RootS3Handler + RejectedS3Handler
{
}

impl<T> S3Handler for T where
    T: ObjectS3Handler + BucketS3Handler + RootS3Handler + RejectedS3Handler
{
}

#[derive(Debug)]
pub struct HandlerError {
    message: String,
}

impl HandlerError {
    fn internal(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}

pub fn router<T, E>(handler: T) -> Router
where
    T: S3Handler
        + ObjectS3Handler<Error = E>
        + BucketS3Handler<Error = E>
        + RootS3Handler<Error = E>
        + RejectedS3Handler<Error = E>
        + Send
        + Sync
        + 'static,
    E: Display + Send + Sync + 'static,
{
    Router::new()
        .route("/s3/dispatch", post(dispatch::<T, E>))
        .with_state(Arc::new(handler))
}

pub async fn dispatch<T, E>(
    State(handler): State<Arc<T>>,
    Json(req): Json<S3Request>,
) -> Result<Json<S3Response>, HandlerError>
where
    T: S3Handler
        + ObjectS3Handler<Error = E>
        + BucketS3Handler<Error = E>
        + RootS3Handler<Error = E>
        + RejectedS3Handler<Error = E>
        + Send
        + Sync,
    E: Display + Send + Sync + 'static,
{
    let resp = match req {
        S3Request::HeadObject(v) => object::head_object::<T, E>(&*handler, v).await,
        S3Request::GetObjectAttributes(v) => {
            object::get_object_attributes::<T, E>(&*handler, v).await
        }
        S3Request::CopyObjectPart(v) => object::copy_object_part::<T, E>(&*handler, v).await,
        S3Request::PutObjectPart(v) => object::put_object_part::<T, E>(&*handler, v).await,
        S3Request::ListObjectParts(v) => object::list_object_parts::<T, E>(&*handler, v).await,
        S3Request::CompleteMultipartUpload(v) => {
            object::complete_multipart_upload::<T, E>(&*handler, v).await
        }
        S3Request::NewMultipartUpload(v) => {
            object::new_multipart_upload::<T, E>(&*handler, v).await
        }
        S3Request::AbortMultipartUpload(v) => {
            object::abort_multipart_upload::<T, E>(&*handler, v).await
        }
        S3Request::GetObjectAcl(v) => object::get_object_acl::<T, E>(&*handler, v).await,
        S3Request::PutObjectAcl(v) => object::put_object_acl::<T, E>(&*handler, v).await,
        S3Request::GetObjectTagging(v) => object::get_object_tagging::<T, E>(&*handler, v).await,
        S3Request::PutObjectTagging(v) => object::put_object_tagging::<T, E>(&*handler, v).await,
        S3Request::DeleteObjectTagging(v) => {
            object::delete_object_tagging::<T, E>(&*handler, v).await
        }
        S3Request::SelectObjectContent(v) => {
            object::select_object_content::<T, E>(&*handler, v).await
        }
        S3Request::GetObjectRetention(v) => {
            object::get_object_retention::<T, E>(&*handler, v).await
        }
        S3Request::GetObjectLegalHold(v) => {
            object::get_object_legal_hold::<T, E>(&*handler, v).await
        }
        S3Request::GetObjectLambda(v) => object::get_object_lambda::<T, E>(&*handler, v).await,
        S3Request::GetObject(v) => object::get_object::<T, E>(&*handler, v).await,
        S3Request::CopyObject(v) => object::copy_object::<T, E>(&*handler, v).await,
        S3Request::PutObjectRetention(v) => {
            object::put_object_retention::<T, E>(&*handler, v).await
        }
        S3Request::PutObjectLegalHold(v) => {
            object::put_object_legal_hold::<T, E>(&*handler, v).await
        }
        S3Request::PutObjectExtract(v) => object::put_object_extract::<T, E>(&*handler, v).await,
        S3Request::AppendObjectRejected(v) => {
            object::append_object_rejected::<T, E>(&*handler, v).await
        }
        S3Request::PutObject(v) => object::put_object::<T, E>(&*handler, v).await,
        S3Request::DeleteObject(v) => object::delete_object::<T, E>(&*handler, v).await,
        S3Request::PostRestoreObject(v) => {
            object::post_restore_object::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketLocation(v) => {
            bucket::get_bucket_location::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketPolicy(v) => bucket::get_bucket_policy::<T, E>(&*handler, v).await,
        S3Request::GetBucketLifecycle(v) => {
            bucket::get_bucket_lifecycle::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketEncryption(v) => {
            bucket::get_bucket_encryption::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketObjectLockConfig(v) => {
            bucket::get_bucket_object_lock_config::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketReplicationConfig(v) => {
            bucket::get_bucket_replication_config::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketVersioning(v) => {
            bucket::get_bucket_versioning::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketNotification(v) => {
            bucket::get_bucket_notification::<T, E>(&*handler, v).await
        }
        S3Request::ListenBucketNotification(v) => {
            bucket::listen_bucket_notification::<T, E>(&*handler, v).await
        }
        S3Request::ResetBucketReplicationStatus(v) => {
            bucket::reset_bucket_replication_status::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketAcl(v) => bucket::get_bucket_acl::<T, E>(&*handler, v).await,
        S3Request::PutBucketAcl(v) => bucket::put_bucket_acl::<T, E>(&*handler, v).await,
        S3Request::GetBucketCors(v) => bucket::get_bucket_cors::<T, E>(&*handler, v).await,
        S3Request::PutBucketCors(v) => bucket::put_bucket_cors::<T, E>(&*handler, v).await,
        S3Request::DeleteBucketCors(v) => bucket::delete_bucket_cors::<T, E>(&*handler, v).await,
        S3Request::GetBucketWebsite(v) => bucket::get_bucket_website::<T, E>(&*handler, v).await,
        S3Request::GetBucketAccelerate(v) => {
            bucket::get_bucket_accelerate::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketRequestPayment(v) => {
            bucket::get_bucket_request_payment::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketLogging(v) => bucket::get_bucket_logging::<T, E>(&*handler, v).await,
        S3Request::GetBucketTagging(v) => bucket::get_bucket_tagging::<T, E>(&*handler, v).await,
        S3Request::DeleteBucketWebsite(v) => {
            bucket::delete_bucket_website::<T, E>(&*handler, v).await
        }
        S3Request::DeleteBucketTagging(v) => {
            bucket::delete_bucket_tagging::<T, E>(&*handler, v).await
        }
        S3Request::ListMultipartUploads(v) => {
            bucket::list_multipart_uploads::<T, E>(&*handler, v).await
        }
        S3Request::ListObjectsV2M(v) => bucket::list_objects_v2m::<T, E>(&*handler, v).await,
        S3Request::ListObjectsV2(v) => bucket::list_objects_v2::<T, E>(&*handler, v).await,
        S3Request::ListObjectVersionsM(v) => {
            bucket::list_object_versions_m::<T, E>(&*handler, v).await
        }
        S3Request::ListObjectVersions(v) => {
            bucket::list_object_versions::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketPolicyStatus(v) => {
            bucket::get_bucket_policy_status::<T, E>(&*handler, v).await
        }
        S3Request::PutBucketLifecycle(v) => {
            bucket::put_bucket_lifecycle::<T, E>(&*handler, v).await
        }
        S3Request::PutBucketReplicationConfig(v) => {
            bucket::put_bucket_replication_config::<T, E>(&*handler, v).await
        }
        S3Request::PutBucketEncryption(v) => {
            bucket::put_bucket_encryption::<T, E>(&*handler, v).await
        }
        S3Request::PutBucketPolicy(v) => bucket::put_bucket_policy::<T, E>(&*handler, v).await,
        S3Request::PutBucketObjectLockConfig(v) => {
            bucket::put_bucket_object_lock_config::<T, E>(&*handler, v).await
        }
        S3Request::PutBucketTagging(v) => bucket::put_bucket_tagging::<T, E>(&*handler, v).await,
        S3Request::PutBucketVersioning(v) => {
            bucket::put_bucket_versioning::<T, E>(&*handler, v).await
        }
        S3Request::PutBucketNotification(v) => {
            bucket::put_bucket_notification::<T, E>(&*handler, v).await
        }
        S3Request::ResetBucketReplicationStart(v) => {
            bucket::reset_bucket_replication_start::<T, E>(&*handler, v).await
        }
        S3Request::PutBucket(v) => bucket::put_bucket::<T, E>(&*handler, v).await,
        S3Request::HeadBucket(v) => bucket::head_bucket::<T, E>(&*handler, v).await,
        S3Request::PostPolicy(v) => bucket::post_policy::<T, E>(&*handler, v).await,
        S3Request::DeleteMultipleObjects(v) => {
            bucket::delete_multiple_objects::<T, E>(&*handler, v).await
        }
        S3Request::DeleteBucketPolicy(v) => {
            bucket::delete_bucket_policy::<T, E>(&*handler, v).await
        }
        S3Request::DeleteBucketReplication(v) => {
            bucket::delete_bucket_replication::<T, E>(&*handler, v).await
        }
        S3Request::DeleteBucketLifecycle(v) => {
            bucket::delete_bucket_lifecycle::<T, E>(&*handler, v).await
        }
        S3Request::DeleteBucketEncryption(v) => {
            bucket::delete_bucket_encryption::<T, E>(&*handler, v).await
        }
        S3Request::DeleteBucket(v) => bucket::delete_bucket::<T, E>(&*handler, v).await,
        S3Request::GetBucketReplicationMetricsV2(v) => {
            bucket::get_bucket_replication_metrics_v2::<T, E>(&*handler, v).await
        }
        S3Request::GetBucketReplicationMetrics(v) => {
            bucket::get_bucket_replication_metrics::<T, E>(&*handler, v).await
        }
        S3Request::ValidateBucketReplicationCreds(v) => {
            bucket::validate_bucket_replication_creds::<T, E>(&*handler, v).await
        }
        S3Request::ListObjectsV1(v) => bucket::list_objects_v1::<T, E>(&*handler, v).await,
        S3Request::RootListenNotification(v) => {
            root::root_listen_notification::<T, E>(&*handler, v).await
        }
        S3Request::ListBuckets(v) => root::list_buckets::<T, E>(&*handler, v).await,
        S3Request::ListBucketsDoubleSlash(v) => {
            root::list_buckets_double_slash::<T, E>(&*handler, v).await
        }
        S3Request::RejectedObjectTorrent(v) => {
            reject::rejected_object_torrent::<T, E>(&*handler, v).await
        }
        S3Request::RejectedObjectAclDelete(v) => {
            reject::rejected_object_acl_delete::<T, E>(&*handler, v).await
        }
        S3Request::RejectedBucketApi(v) => reject::rejected_bucket_api::<T, E>(&*handler, v).await,
    }
    .map_err(|e| HandlerError::internal(e.to_string()))?;

    Ok(Json(resp))
}
