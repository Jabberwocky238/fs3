use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Method};
use axum::routing::any;
use axum::Router;

use crate::types::s3::core::ObjectAttribute;
use crate::types::s3::request::*;
use crate::types::s3::response::S3Response;
use crate::types::traits::s3_engine::S3EngineError;
use crate::types::traits::s3_handler::{S3Handler, S3HandlerBridgeError};

use super::util::{body_string, get, has, header, header_eq, multipart_selector};
use super::{HandlerError, ObjectError};

fn object_err(e: impl std::fmt::Display) -> HandlerError {
    let msg = e.to_string();
    HandlerError::Object(if msg.contains("object not found") || msg.contains("version not found") {
        ObjectError::NotFound(msg)
    } else if msg.contains("multipart") && msg.contains("not found") {
        ObjectError::UploadNotFound(msg)
    } else {
        ObjectError::Internal(msg)
    })
}

pub fn routes<T, E>(state: Arc<T>) -> Router
where
    T: S3Handler<E> + Send + Sync + 'static,
    E: S3EngineError + From<S3HandlerBridgeError>,
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
) -> Result<S3Response, HandlerError>
where
    T: S3Handler<E> + Send + Sync,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    let mk = || ObjectRef { bucket: bucket.clone(), object: object_path.clone() };
    if has(&q, "torrent") && matches!(method, Method::GET | Method::PUT | Method::DELETE) {
        let v = handler
            .rejected_object_torrent(RejectedObjectTorrentRequest {
                object: mk(),
                method: method.to_string(),
            })
            .await
            .map_err(object_err)?;
        return Ok(S3Response::RejectedApi(v));
    }
    if has(&q, "acl") && method == Method::DELETE {
        let v = handler
            .rejected_object_acl_delete(RejectedObjectAclDeleteRequest { object: mk() })
            .await
            .map_err(object_err)?;
        return Ok(S3Response::RejectedApi(v));
    }

    let text = String::from_utf8_lossy(&body).to_string();
    let copy_source = header(&headers, "x-amz-copy-source");
    let resp = match method {
        Method::HEAD => S3Response::HeadObject(
            handler.head_object(HeadObjectRequest {
                object: mk(),
                range: header(&headers, "range"),
                version_id: get(&q, "versionId"),
                if_match: header(&headers, "if-match"),
                if_none_match: header(&headers, "if-none-match"),
            }).await.map_err(object_err)?,
        ),
        Method::GET if has(&q, "attributes") => S3Response::GetObjectAttributes(
            handler.get_object_attributes(GetObjectAttributesRequest {
                object: mk(),
                attributes: get(&q, "attributes")
                    .map(|v| {
                        v.split(',')
                            .map(|x| x.trim().to_ascii_lowercase())
                            .filter_map(|x| match x.as_str() {
                                "etag" => Some(ObjectAttribute::ETag),
                                "checksum" => Some(ObjectAttribute::Checksum),
                                "objectparts" | "object_parts" | "parts" => Some(ObjectAttribute::ObjectParts),
                                "storageclass" | "storage_class" => Some(ObjectAttribute::StorageClass),
                                "objectsize" | "object_size" | "size" => Some(ObjectAttribute::ObjectSize),
                                "lastmodified" | "last_modified" => Some(ObjectAttribute::LastModified),
                                _ => None,
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            }).await.map_err(object_err)?,
        ),
        Method::GET if has(&q, "uploadId") => S3Response::ListObjectParts(
            handler.list_object_parts(ListObjectPartsRequest {
                object: mk(),
                upload_id: get(&q, "uploadId").unwrap_or_default(),
            }).await.map_err(object_err)?,
        ),
        Method::GET if has(&q, "acl") => S3Response::GetObjectAcl(
            handler.get_object_acl(GetObjectAclRequest { object: mk() }).await.map_err(object_err)?,
        ),
        Method::GET if has(&q, "tagging") => S3Response::GetObjectTagging(
            handler.get_object_tagging(GetObjectTaggingRequest { object: mk() }).await.map_err(object_err)?,
        ),
        Method::GET if has(&q, "retention") => S3Response::GetObjectRetention(
            handler.get_object_retention(GetObjectRetentionRequest { object: mk() }).await.map_err(object_err)?,
        ),
        Method::GET if has(&q, "legal-hold") => S3Response::GetObjectLegalHold(
            handler.get_object_legal_hold(GetObjectLegalHoldRequest { object: mk() }).await.map_err(object_err)?,
        ),
        Method::GET if has(&q, "lambdaArn") => S3Response::GetObjectLambda(
            handler.get_object_lambda(GetObjectLambdaRequest {
                object: mk(),
                lambda_arn: get(&q, "lambdaArn").unwrap_or_default(),
                range: header(&headers, "range"),
                version_id: get(&q, "versionId"),
                if_match: header(&headers, "if-match"),
                if_none_match: header(&headers, "if-none-match"),
            }).await.map_err(object_err)?,
        ),
        Method::GET => S3Response::GetObject(
            handler.get_object(GetObjectRequest {
                object: mk(),
                range: header(&headers, "range"),
                version_id: get(&q, "versionId"),
            }).await.map_err(object_err)?,
        ),

        Method::PUT if has(&q, "uploadId") && has(&q, "partNumber") && copy_source.is_some() => S3Response::CopyObjectPart(
            handler.copy_object_part(CopyObjectPartRequest {
                object: mk(),
                multipart: multipart_selector(&q),
                copy_source: copy_source.unwrap_or_default(),
            }).await.map_err(object_err)?,
        ),
        Method::PUT if has(&q, "uploadId") && has(&q, "partNumber") => S3Response::PutObjectPart(
            handler.put_object_part(PutObjectPartRequest {
                object: mk(),
                multipart: multipart_selector(&q),
                body: body.to_vec(),
                checksum: header(&headers, "x-amz-checksum-sha256"),
            }).await.map_err(object_err)?,
        ),
        Method::PUT if has(&q, "acl") => S3Response::PutObjectAcl(
            handler.put_object_acl(PutObjectAclRequest { object: mk(), xml: body_string(&body) }).await.map_err(object_err)?,
        ),
        Method::PUT if has(&q, "tagging") => S3Response::PutObjectTagging(
            handler.put_object_tagging(PutObjectTaggingRequest { object: mk(), xml: text }).await.map_err(object_err)?,
        ),
        Method::PUT if has(&q, "retention") => S3Response::PutObjectRetention(
            handler.put_object_retention(PutObjectRetentionRequest { object: mk(), xml: text }).await.map_err(object_err)?,
        ),
        Method::PUT if has(&q, "legal-hold") => S3Response::PutObjectLegalHold(
            handler.put_object_legal_hold(PutObjectLegalHoldRequest { object: mk(), xml: text }).await.map_err(object_err)?,
        ),
        Method::PUT if header_eq(&headers, "x-amz-snowball-extract", "true") => S3Response::PutObjectExtract(
            handler.put_object_extract(PutObjectExtractRequest { object: mk(), body: body.to_vec() }).await.map_err(object_err)?,
        ),
        Method::PUT if header(&headers, "x-amz-write-offset-bytes").is_some() => S3Response::AppendObjectRejected(
            handler.append_object_rejected(AppendObjectRejectedRequest {
                object: mk(),
                write_offset_bytes: header(&headers, "x-amz-write-offset-bytes").unwrap_or_default(),
                body: body.to_vec(),
            }).await.map_err(object_err)?,
        ),
        Method::PUT if copy_source.is_some() => S3Response::CopyObject(
            handler.copy_object(CopyObjectRequest {
                object: mk(),
                copy_source: copy_source.unwrap_or_default(),
                metadata_directive: header(&headers, "x-amz-metadata-directive"),
            }).await.map_err(object_err)?,
        ),
        Method::PUT => S3Response::PutObject(
            handler.put_object(PutObjectRequest { object: mk(), body: body.to_vec(), content_type: header(&headers, "content-type") }).await.map_err(object_err)?,
        ),

        Method::POST if has(&q, "uploadId") => S3Response::CompleteMultipartUpload(
            handler.complete_multipart_upload(CompleteMultipartUploadRequest {
                object: mk(),
                upload_id: get(&q, "uploadId").unwrap_or_default(),
                xml: text,
            }).await.map_err(object_err)?,
        ),
        Method::POST if has(&q, "uploads") => S3Response::NewMultipartUpload(
            handler.new_multipart_upload(NewMultipartUploadRequest { object: mk() }).await.map_err(object_err)?,
        ),
        Method::POST if has(&q, "select") && get(&q, "select-type").as_deref() == Some("2") => S3Response::SelectObjectContent(
            handler.select_object_content(SelectObjectContentRequest { object: mk(), select_type: 2, xml: text }).await.map_err(object_err)?,
        ),
        Method::POST if has(&q, "restore") => S3Response::PostRestoreObject(
            handler.post_restore_object(PostRestoreObjectRequest { object: mk(), xml: text }).await.map_err(object_err)?,
        ),

        Method::DELETE if has(&q, "uploadId") => S3Response::AbortMultipartUpload(
            handler.abort_multipart_upload(AbortMultipartUploadRequest { object: mk(), upload_id: get(&q, "uploadId").unwrap_or_default() }).await.map_err(object_err)?,
        ),
        Method::DELETE if has(&q, "tagging") => S3Response::DeleteObjectTagging(
            handler.delete_object_tagging(DeleteObjectTaggingRequest { object: mk() }).await.map_err(object_err)?,
        ),
        Method::DELETE => S3Response::DeleteObject(
            handler.delete_object(DeleteObjectRequest { object: mk(), version_id: get(&q, "versionId") }).await.map_err(object_err)?,
        ),
        _ => return Err(HandlerError::method_not_allowed("unsupported object API")),
    };
    Ok(resp)
}
