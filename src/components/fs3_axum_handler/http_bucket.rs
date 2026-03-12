use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Method};
use axum::routing::any;
use axum::Router;

use crate::types::FS3Error;
use crate::types::s3::request::*;
use crate::types::s3::response::S3Response;
use crate::types::traits::s3_handler::S3Handler;

use super::util::{body_text, get, has, header, list_query};

pub fn routes<T>(state: Arc<T>) -> Router
where
    T: S3Handler + Send + Sync + 'static,
{
    Router::new()
        .route("/{bucket}", any(bucket_entry::<T>))
        .route("/{bucket}/", any(bucket_entry::<T>))
        .with_state(state)
}

async fn bucket_entry<T>(
    State(handler): State<Arc<T>>,
    Path(bucket_name): Path<String>,
    method: Method,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    body: Body,
) -> Result<S3Response, FS3Error>
where
    T: S3Handler + Send + Sync,
{
    let mk = || BucketRef { bucket: bucket_name.clone() };

    let resp = match method {
        Method::GET if has(&q, "location") => S3Response::GetBucketLocation(handler.get_bucket_location(GetBucketLocationRequest { bucket: mk() }).await?),
        Method::GET if has(&q, "policy") => S3Response::GetBucketPolicy(handler.get_bucket_policy(GetBucketPolicyRequest { bucket: mk() }).await?),
        Method::GET if has(&q, "uploads") => S3Response::ListMultipartUploads(handler.list_multipart_uploads(ListMultipartUploadsRequest { bucket: mk(), query: list_query(&q) }).await?),
        Method::GET if get(&q, "list-type").as_deref() == Some("2") && get(&q, "metadata").as_deref() == Some("true") => S3Response::ListObjectsV2M(handler.list_objects_v2m(ListObjectsV2MRequest { bucket: mk(), query: list_query(&q), metadata: true }).await?),
        Method::GET if get(&q, "list-type").as_deref() == Some("2") => S3Response::ListObjectsV2(handler.list_objects_v2(ListObjectsV2Request { bucket: mk(), query: list_query(&q) }).await?),
        Method::GET if has(&q, "versions") && get(&q, "metadata").as_deref() == Some("true") => S3Response::ListObjectVersionsM(handler.list_object_versions_m(ListObjectVersionsMRequest { bucket: mk(), query: list_query(&q), metadata: true }).await?),
        Method::GET if has(&q, "versions") => S3Response::ListObjectVersions(handler.list_object_versions(ListObjectVersionsRequest { bucket: mk(), query: list_query(&q) }).await?),
        Method::GET if has(&q, "policyStatus") => S3Response::GetBucketPolicyStatus(handler.get_bucket_policy_status(GetBucketPolicyStatusRequest { bucket: mk() }).await?),
        Method::GET => S3Response::ListObjectsV1(handler.list_objects_v1(ListObjectsV1Request { bucket: mk(), query: list_query(&q) }).await?),

        Method::PUT if has(&q, "policy") => {
            let json = body_text(body).await?;
            S3Response::PutBucketPolicy(handler.put_bucket_policy(PutBucketPolicyRequest { bucket: mk(), json }).await?)
        }
        Method::PUT => S3Response::PutBucket(handler.put_bucket(PutBucketRequest { bucket: mk(), region: header(&headers, "x-amz-bucket-region") }).await?),

        Method::POST if has(&q, "delete") => {
            let xml = body_text(body).await?;
            let payload = crate::types::s3::xml::parse_delete_objects(&xml)?;
            S3Response::DeleteMultipleObjects(handler.delete_multiple_objects(DeleteMultipleObjectsRequest {
                bucket: mk(),
                payload: DeleteObjectsInput { quiet: payload.quiet, keys: payload.keys },
            }).await?)
        }

        Method::DELETE if has(&q, "policy") => S3Response::DeleteBucketPolicy(handler.delete_bucket_policy(DeleteBucketPolicyRequest { bucket: mk() }).await?),
        Method::DELETE => S3Response::DeleteBucket(handler.delete_bucket(DeleteBucketRequest { bucket: mk() }).await?),

        Method::HEAD => S3Response::HeadBucket(handler.head_bucket(HeadBucketRequest { bucket: mk() }).await?),
        _ => return Err(FS3Error::method_not_allowed("unsupported bucket API")),
    };

    Ok(resp)
}
