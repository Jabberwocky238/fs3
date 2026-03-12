use std::collections::HashMap;
use std::io;
use std::pin::Pin;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Method};
use axum::routing::any;
use axum::Router;
use futures::StreamExt as _;

use crate::types::s3::core::ObjectAttribute;
use crate::types::s3::request::*;
use crate::types::s3::response::S3Response;
use crate::types::s3::xml;
use crate::types::errors::S3EngineError;
use crate::types::traits::s3_handler::{S3Handler, S3HandlerBridgeError};

use super::util::{get, has, header, header_eq, multipart_selector};
use super::{HandlerError, ObjectError};

fn object_err(e: impl std::fmt::Display) -> HandlerError {
    let msg = e.to_string();
    HandlerError::Object(if msg.contains("object not found") || msg.contains("version not found") {
        ObjectError::NotFound(msg)
    } else if msg.contains("multipart") && msg.contains("not found") {
        ObjectError::UploadNotFound(msg)
    } else if msg.contains("precondition failed") {
        ObjectError::PreconditionFailed(msg)
    } else if msg.contains("not modified") {
        ObjectError::NotModified(msg)
    } else {
        ObjectError::Internal(msg)
    })
}

pub fn routes<T, E>(state: Arc<T>) -> Router
where
    T: S3Handler<E> + Send + Sync + 'static,
    E: std::fmt::Display + From<S3HandlerBridgeError> + From<S3EngineError> + 'static,
{
    Router::new().route("/{bucket}/{*object}", any(object_entry::<T, E>)).with_state(state)
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

async fn object_entry<T, E>(
    State(handler): State<Arc<T>>,
    Path((bucket, object_path)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    body: Body,
) -> Result<S3Response, HandlerError>
where
    T: S3Handler<E> + Send + Sync,
    E: std::fmt::Display + From<S3HandlerBridgeError> + From<S3EngineError> + 'static,
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
            handler.get_object_retention(GetObjectRetentionRequest { bucket: BucketRef { bucket: bucket.clone() }, object: mk() }).await.map_err(object_err)?,
        ),
        Method::GET if has(&q, "legal-hold") => S3Response::GetObjectLegalHold(
            handler.get_object_legal_hold(GetObjectLegalHoldRequest { bucket: BucketRef { bucket: bucket.clone() }, object: mk() }).await.map_err(object_err)?,
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
                if_match: header(&headers, "if-match"),
                if_none_match: header(&headers, "if-none-match"),
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
                body: body_stream(body),
                checksum: header(&headers, "x-amz-checksum-sha256"),
            }).await.map_err(object_err)?,
        ),
        Method::PUT if has(&q, "acl") => {
            let xml = body_text(body).await.map_err(object_err)?;
            let acl = if xml.trim().is_empty() {
                None
            } else {
                Some(xml::parse_access_control_policy(&xml).map_err(object_err)?)
            };
            S3Response::PutObjectAcl(
                handler.put_object_acl(PutObjectAclRequest { object: mk(), acl }).await.map_err(object_err)?,
            )
        }
        Method::PUT if has(&q, "tagging") => {
            let xml = body_text(body).await.map_err(object_err)?;
            let tags = xml::parse_tagging(&xml).map_err(object_err)?;
            S3Response::PutObjectTagging(
                handler.put_object_tagging(PutObjectTaggingRequest { object: mk(), tags }).await.map_err(object_err)?,
            )
        }
        Method::PUT if has(&q, "retention") => {
            let xml = body_text(body).await.map_err(object_err)?;
            let retention = xml::parse_retention(&xml).map_err(object_err)?;
            S3Response::PutObjectRetention(
                handler.put_object_retention(PutObjectRetentionRequest {
                    bucket: BucketRef { bucket: bucket.clone() },
                    object: mk(),
                    retention,
                }).await.map_err(object_err)?,
            )
        }
        Method::PUT if has(&q, "legal-hold") => {
            let xml = body_text(body).await.map_err(object_err)?;
            let legal_hold = xml::parse_legal_hold(&xml).map_err(object_err)?;
            S3Response::PutObjectLegalHold(
                handler.put_object_legal_hold(PutObjectLegalHoldRequest {
                    bucket: BucketRef { bucket: bucket.clone() },
                    object: mk(),
                    legal_hold,
                }).await.map_err(object_err)?,
            )
        }
        Method::PUT if header_eq(&headers, "x-amz-snowball-extract", "true") => S3Response::PutObjectExtract(
            handler.put_object_extract(PutObjectExtractRequest { object: mk(), body: body_stream(body) }).await.map_err(object_err)?,
        ),
        Method::PUT if header(&headers, "x-amz-write-offset-bytes").is_some() => S3Response::AppendObjectRejected(
            handler.append_object_rejected(AppendObjectRejectedRequest {
                object: mk(),
                write_offset_bytes: header(&headers, "x-amz-write-offset-bytes").unwrap_or_default(),
                body: body_stream(body),
            }).await.map_err(object_err)?,
        ),
        Method::PUT if copy_source.is_some() => S3Response::CopyObject(
            handler.copy_object(CopyObjectRequest {
                object: mk(),
                copy_source: copy_source.unwrap_or_default(),
                metadata_directive: header(&headers, "x-amz-metadata-directive"),
            }).await.map_err(object_err)?,
        ),
        Method::PUT => {
            let content_length = header(&headers, "content-length").and_then(|v| v.parse::<u64>().ok());
            let mut user_metadata = HashMap::new();
            for (k, v) in headers.iter() {
                if let Some(key) = k.as_str().strip_prefix("x-amz-meta-") {
                    if let Ok(val) = v.to_str() {
                        user_metadata.insert(key.to_string(), val.to_string());
                    }
                }
            }
            S3Response::PutObject(
                handler.put_object(PutObjectRequest {
                    object: mk(),
                    body: body_stream(body),
                    content_type: header(&headers, "content-type"),
                    content_md5: header(&headers, "content-md5"),
                    content_length,
                    user_metadata,
                }).await.map_err(object_err)?,
            )
        },

        Method::POST if has(&q, "uploadId") => {
            let xml = body_text(body).await.map_err(object_err)?;
            let completed = xml::parse_complete_multipart_upload(&xml).map_err(object_err)?;
            S3Response::CompleteMultipartUpload(
                handler.complete_multipart_upload(CompleteMultipartUploadRequest {
                    object: mk(),
                    upload_id: get(&q, "uploadId").unwrap_or_default(),
                    completed,
                }).await.map_err(object_err)?,
            )
        }
        Method::POST if has(&q, "uploads") => S3Response::NewMultipartUpload(
            handler.new_multipart_upload(NewMultipartUploadRequest { object: mk() }).await.map_err(object_err)?,
        ),
        Method::POST if has(&q, "select") && get(&q, "select-type").as_deref() == Some("2") => {
            let xml = body_text(body).await.map_err(object_err)?;
            let input = xml::parse_select_object_content(&xml).map_err(object_err)?;
            S3Response::SelectObjectContent(
                handler.select_object_content(SelectObjectContentRequest {
                    object: mk(),
                    select_type: 2,
                    input,
                }).await.map_err(object_err)?,
            )
        }
        Method::POST if has(&q, "restore") => {
            let xml = body_text(body).await.map_err(object_err)?;
            let restore = xml::parse_restore_object(&xml).map_err(object_err)?;
            S3Response::PostRestoreObject(
                handler.post_restore_object(PostRestoreObjectRequest {
                    object: mk(),
                    restore,
                }).await.map_err(object_err)?,
            )
        }

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
