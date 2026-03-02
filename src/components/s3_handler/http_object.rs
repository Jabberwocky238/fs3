use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Method};
use axum::routing::any;
use axum::{Json, Router};

use crate::types::s3::request::*;
use crate::types::s3::response::S3Response;
use crate::types::traits::s3_handler::{ObjectS3Handler, RejectedS3Handler};

use super::util::{body_string, get, has, header, header_eq, multipart_selector};
use super::{object, reject, HandlerError, S3Handler};

pub fn routes<T, E>(state: Arc<T>) -> Router
where
    T: S3Handler + ObjectS3Handler<Error = E> + RejectedS3Handler<Error = E> + Send + Sync + 'static,
    E: Display + Send + Sync + 'static,
{
    Router::new().route("/{bucket}/{*object}", any(object_entry::<T, E>)).with_state(state)
}

async fn object_entry<T, E>(
    State(handler): State<Arc<T>>,
    Path((bucket, object_path)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<Json<S3Response>, HandlerError>
where
    T: S3Handler + ObjectS3Handler<Error = E> + RejectedS3Handler<Error = E> + Send + Sync,
    E: Display + Send + Sync + 'static,
{
    let mk = || ObjectRef { bucket: bucket.clone(), object: object_path.clone() };
    if has(&q, "torrent") && matches!(method, Method::GET | Method::PUT | Method::DELETE) {
        let resp = reject::rejected_object_torrent::<T, E>(&*handler, RejectedObjectTorrentRequest {
            object: mk(),
            method: method.to_string(),
        }).await.map_err(|e| HandlerError::internal(e.to_string()))?;
        return Ok(Json(resp));
    }
    if has(&q, "acl") && method == Method::DELETE {
        let resp = reject::rejected_object_acl_delete::<T, E>(&*handler, RejectedObjectAclDeleteRequest { object: mk() })
            .await.map_err(|e| HandlerError::internal(e.to_string()))?;
        return Ok(Json(resp));
    }

    let text = String::from_utf8_lossy(&body).to_string();
    let copy_source = header(&headers, "x-amz-copy-source");
    let resp = match method {
        Method::HEAD => object::head_object::<T, E>(&*handler, HeadObjectRequest { object: mk() }).await,
        Method::GET if has(&q, "attributes") => object::get_object_attributes::<T, E>(&*handler, GetObjectAttributesRequest {
            object: mk(),
            attributes: get(&q, "attributes").map(|v| v.split(',').map(|x| x.to_string()).collect()).unwrap_or_default(),
        }).await,
        Method::GET if has(&q, "uploadId") => object::list_object_parts::<T, E>(&*handler, ListObjectPartsRequest {
            object: mk(),
            upload_id: get(&q, "uploadId").unwrap_or_default(),
        }).await,
        Method::GET if has(&q, "acl") => object::get_object_acl::<T, E>(&*handler, GetObjectAclRequest { object: mk() }).await,
        Method::GET if has(&q, "tagging") => object::get_object_tagging::<T, E>(&*handler, GetObjectTaggingRequest { object: mk() }).await,
        Method::GET if has(&q, "retention") => object::get_object_retention::<T, E>(&*handler, GetObjectRetentionRequest { object: mk() }).await,
        Method::GET if has(&q, "legal-hold") => object::get_object_legal_hold::<T, E>(&*handler, GetObjectLegalHoldRequest { object: mk() }).await,
        Method::GET if has(&q, "lambdaArn") => object::get_object_lambda::<T, E>(&*handler, GetObjectLambdaRequest {
            object: mk(),
            lambda_arn: get(&q, "lambdaArn").unwrap_or_default(),
        }).await,
        Method::GET => object::get_object::<T, E>(&*handler, GetObjectRequest {
            object: mk(),
            range: header(&headers, "range"),
            version_id: get(&q, "versionId"),
        }).await,

        Method::PUT if has(&q, "uploadId") && has(&q, "partNumber") && copy_source.is_some() => object::copy_object_part::<T, E>(&*handler, CopyObjectPartRequest {
            object: mk(),
            multipart: multipart_selector(&q),
            copy_source: copy_source.unwrap_or_default(),
        }).await,
        Method::PUT if has(&q, "uploadId") && has(&q, "partNumber") => object::put_object_part::<T, E>(&*handler, PutObjectPartRequest {
            object: mk(),
            multipart: multipart_selector(&q),
            body: body.to_vec(),
            checksum: header(&headers, "x-amz-checksum-sha256"),
        }).await,
        Method::PUT if has(&q, "acl") => object::put_object_acl::<T, E>(&*handler, PutObjectAclRequest { object: mk(), xml: body_string(&body) }).await,
        Method::PUT if has(&q, "tagging") => object::put_object_tagging::<T, E>(&*handler, PutObjectTaggingRequest { object: mk(), xml: text }).await,
        Method::PUT if has(&q, "retention") => object::put_object_retention::<T, E>(&*handler, PutObjectRetentionRequest { object: mk(), xml: text }).await,
        Method::PUT if has(&q, "legal-hold") => object::put_object_legal_hold::<T, E>(&*handler, PutObjectLegalHoldRequest { object: mk(), xml: text }).await,
        Method::PUT if header_eq(&headers, "x-amz-snowball-extract", "true") => object::put_object_extract::<T, E>(&*handler, PutObjectExtractRequest { object: mk(), body: body.to_vec() }).await,
        Method::PUT if header(&headers, "x-amz-write-offset-bytes").is_some() => object::append_object_rejected::<T, E>(&*handler, AppendObjectRejectedRequest {
            object: mk(),
            write_offset_bytes: header(&headers, "x-amz-write-offset-bytes").unwrap_or_default(),
            body: body.to_vec(),
        }).await,
        Method::PUT if copy_source.is_some() => object::copy_object::<T, E>(&*handler, CopyObjectRequest {
            object: mk(),
            copy_source: copy_source.unwrap_or_default(),
            metadata_directive: header(&headers, "x-amz-metadata-directive"),
        }).await,
        Method::PUT => object::put_object::<T, E>(&*handler, PutObjectRequest { object: mk(), body: body.to_vec(), content_type: header(&headers, "content-type") }).await,

        Method::POST if has(&q, "uploadId") => object::complete_multipart_upload::<T, E>(&*handler, CompleteMultipartUploadRequest {
            object: mk(),
            upload_id: get(&q, "uploadId").unwrap_or_default(),
            xml: text,
        }).await,
        Method::POST if has(&q, "uploads") => object::new_multipart_upload::<T, E>(&*handler, NewMultipartUploadRequest { object: mk() }).await,
        Method::POST if has(&q, "select") && get(&q, "select-type").as_deref() == Some("2") => object::select_object_content::<T, E>(&*handler, SelectObjectContentRequest { object: mk(), select_type: 2, xml: text }).await,
        Method::POST if has(&q, "restore") => object::post_restore_object::<T, E>(&*handler, PostRestoreObjectRequest { object: mk(), xml: text }).await,

        Method::DELETE if has(&q, "uploadId") => object::abort_multipart_upload::<T, E>(&*handler, AbortMultipartUploadRequest { object: mk(), upload_id: get(&q, "uploadId").unwrap_or_default() }).await,
        Method::DELETE if has(&q, "tagging") => object::delete_object_tagging::<T, E>(&*handler, DeleteObjectTaggingRequest { object: mk() }).await,
        Method::DELETE => object::delete_object::<T, E>(&*handler, DeleteObjectRequest { object: mk(), version_id: get(&q, "versionId") }).await,
        _ => return Err(HandlerError::method_not_allowed("unsupported object API")),
    }.map_err(|e| HandlerError::internal(e.to_string()))?;

    Ok(Json(resp))
}
