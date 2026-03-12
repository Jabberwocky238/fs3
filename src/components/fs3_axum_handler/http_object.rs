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

use crate::types::errors::FS3Error;
use crate::types::s3::core::ObjectAttribute;
use crate::types::s3::request::*;
use crate::types::s3::response::S3Response;
use crate::types::s3::xml;
use crate::types::traits::s3_handler::S3Handler;

use super::util::{get, has, header, header_eq, multipart_selector, body_stream, body_text};

pub fn routes<T>(state: Arc<T>) -> Router
where
    T: S3Handler + Send + Sync + 'static,
{
    Router::new().route("/{bucket}/{*object}", any(object_entry::<T>)).with_state(state)
}

async fn object_entry<T>(
    State(handler): State<Arc<T>>,
    Path((bucket, object_path)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    body: Body,
) -> Result<S3Response, FS3Error>
where
    T: S3Handler + Send + Sync,
{
    let mk = || ObjectRef { bucket: bucket.clone(), object: object_path.clone() };
    if has(&q, "torrent") && matches!(method, Method::GET | Method::PUT | Method::DELETE) {
        let v = handler
            .rejected_object_torrent(RejectedObjectTorrentRequest {
                object: mk(),
                method: method.to_string(),
            })
            .await?;
        return Ok(S3Response::RejectedApi(v));
    }
    if has(&q, "acl") && method == Method::DELETE {
        let v = handler
            .rejected_object_acl_delete(RejectedObjectAclDeleteRequest { object: mk() })
            .await
            ?;
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
            }).await?,
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
            }).await?,
        ),
        Method::GET if has(&q, "uploadId") => S3Response::ListObjectParts(
            handler.list_object_parts(ListObjectPartsRequest {
                object: mk(),
                upload_id: get(&q, "uploadId").unwrap_or_default(),
            }).await?,
        ),
        Method::GET if has(&q, "acl") => S3Response::GetObjectAcl(
            handler.get_object_acl(GetObjectAclRequest { object: mk() }).await?,
        ),
        Method::GET if has(&q, "tagging") => S3Response::GetObjectTagging(
            handler.get_object_tagging(GetObjectTaggingRequest { object: mk() }).await?,
        ),
        Method::GET if has(&q, "retention") => S3Response::GetObjectRetention(
            handler.get_object_retention(GetObjectRetentionRequest { bucket: BucketRef { bucket: bucket.clone() }, object: mk() }).await?,
        ),
        Method::GET if has(&q, "legal-hold") => S3Response::GetObjectLegalHold(
            handler.get_object_legal_hold(GetObjectLegalHoldRequest { bucket: BucketRef { bucket: bucket.clone() }, object: mk() }).await?,
        ),
        Method::GET if has(&q, "lambdaArn") => S3Response::GetObjectLambda(
            handler.get_object_lambda(GetObjectLambdaRequest {
                object: mk(),
                lambda_arn: get(&q, "lambdaArn").unwrap_or_default(),
                range: header(&headers, "range"),
                version_id: get(&q, "versionId"),
                if_match: header(&headers, "if-match"),
                if_none_match: header(&headers, "if-none-match"),
            }).await?,
        ),
        Method::GET => S3Response::GetObject(
            handler.get_object(GetObjectRequest {
                object: mk(),
                range: header(&headers, "range"),
                version_id: get(&q, "versionId"),
                if_match: header(&headers, "if-match"),
                if_none_match: header(&headers, "if-none-match"),
            }).await?,
        ),

        Method::PUT if has(&q, "uploadId") && has(&q, "partNumber") && copy_source.is_some() => S3Response::CopyObjectPart(
            handler.copy_object_part(CopyObjectPartRequest {
                object: mk(),
                multipart: multipart_selector(&q),
                copy_source: copy_source.unwrap_or_default(),
            }).await?,
        ),
        Method::PUT if has(&q, "uploadId") && has(&q, "partNumber") => S3Response::PutObjectPart(
            handler.put_object_part(PutObjectPartRequest {
                object: mk(),
                multipart: multipart_selector(&q),
                body: body_stream(body),
                checksum: header(&headers, "x-amz-checksum-sha256"),
                content_md5: header(&headers, "content-md5"),
                content_encoding: header(&headers, "content-encoding"),
                amz_content_sha256: header(&headers, "x-amz-content-sha256"),
                decoded_content_length: header(&headers, "x-amz-decoded-content-length"),
                amz_trailer: header(&headers, "x-amz-trailer"),
            }).await?,
        ),
        Method::PUT if has(&q, "acl") => {
            let xml = body_text(body).await?;
            let acl = if xml.trim().is_empty() {
                None
            } else {
                Some(xml::parse_access_control_policy(&xml)?)
            };
            S3Response::PutObjectAcl(
                handler.put_object_acl(PutObjectAclRequest { object: mk(), acl }).await?,
            )
        }
        Method::PUT if has(&q, "tagging") => {
            let xml = body_text(body).await?;
            let tags = xml::parse_tagging(&xml)?;
            S3Response::PutObjectTagging(
                handler.put_object_tagging(PutObjectTaggingRequest { object: mk(), tags }).await?,
            )
        }
        Method::PUT if has(&q, "retention") => {
            let xml = body_text(body).await?;
            let retention = xml::parse_retention(&xml)?;
            S3Response::PutObjectRetention(
                handler.put_object_retention(PutObjectRetentionRequest {
                    bucket: BucketRef { bucket: bucket.clone() },
                    object: mk(),
                    retention,
                }).await?,
            )
        }
        Method::PUT if has(&q, "legal-hold") => {
            let xml = body_text(body).await?;
            let legal_hold = xml::parse_legal_hold(&xml)?;
            S3Response::PutObjectLegalHold(
                handler.put_object_legal_hold(PutObjectLegalHoldRequest {
                    bucket: BucketRef { bucket: bucket.clone() },
                    object: mk(),
                    legal_hold,
                }).await?,
            )
        }
        Method::PUT if header_eq(&headers, "x-amz-snowball-extract", "true") => S3Response::PutObjectExtract(
            handler.put_object_extract(PutObjectExtractRequest { object: mk(), body: body_stream(body) }).await?,
        ),
        Method::PUT if header(&headers, "x-amz-write-offset-bytes").is_some() => S3Response::AppendObjectRejected(
            handler.append_object_rejected(AppendObjectRejectedRequest {
                object: mk(),
                write_offset_bytes: header(&headers, "x-amz-write-offset-bytes").unwrap_or_default(),
                body: body_stream(body),
            }).await?,
        ),
        Method::PUT if copy_source.is_some() => S3Response::CopyObject(
            handler.copy_object(CopyObjectRequest {
                object: mk(),
                copy_source: copy_source.unwrap_or_default(),
                metadata_directive: header(&headers, "x-amz-metadata-directive"),
            }).await?,
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
                    checksum_sha256: header(&headers, "x-amz-checksum-sha256"),
                    checksum_sha1: header(&headers, "x-amz-checksum-sha1"),
                    checksum_crc32: header(&headers, "x-amz-checksum-crc32"),
                    checksum_crc32c: header(&headers, "x-amz-checksum-crc32c"),
                    content_length,
                    content_encoding: header(&headers, "content-encoding"),
                    amz_content_sha256: header(&headers, "x-amz-content-sha256"),
                    decoded_content_length: header(&headers, "x-amz-decoded-content-length"),
                    amz_trailer: header(&headers, "x-amz-trailer"),
                    sse: header(&headers, "x-amz-server-side-encryption"),
                    sse_customer_algorithm: header(&headers, "x-amz-server-side-encryption-customer-algorithm"),
                    sse_customer_key: header(&headers, "x-amz-server-side-encryption-customer-key"),
                    sse_customer_key_md5: header(&headers, "x-amz-server-side-encryption-customer-key-md5"),
                    sse_kms_key_id: header(&headers, "x-amz-server-side-encryption-aws-kms-key-id"),
                    sse_context: header(&headers, "x-amz-server-side-encryption-context"),
                    user_metadata,
                }).await?,
            )
        },

        Method::POST if has(&q, "uploadId") => {
            let xml = body_text(body).await?;
            let completed = xml::parse_complete_multipart_upload(&xml)?;
            S3Response::CompleteMultipartUpload(
                handler.complete_multipart_upload(CompleteMultipartUploadRequest {
                    object: mk(),
                    upload_id: get(&q, "uploadId").unwrap_or_default(),
                    completed,
                }).await?,
            )
        }
        Method::POST if has(&q, "uploads") => S3Response::NewMultipartUpload(
            handler.new_multipart_upload(NewMultipartUploadRequest { object: mk() }).await?,
        ),
        Method::POST if has(&q, "select") && get(&q, "select-type").as_deref() == Some("2") => {
            let xml = body_text(body).await?;
            let input = xml::parse_select_object_content(&xml)?;
            S3Response::SelectObjectContent(
                handler.select_object_content(SelectObjectContentRequest {
                    object: mk(),
                    select_type: 2,
                    input,
                }).await?,
            )
        }
        Method::POST if has(&q, "restore") => {
            let xml = body_text(body).await?;
            let restore = xml::parse_restore_object(&xml)?;
            S3Response::PostRestoreObject(
                handler.post_restore_object(PostRestoreObjectRequest {
                    object: mk(),
                    restore,
                }).await?,
            )
        }

        Method::DELETE if has(&q, "uploadId") => S3Response::AbortMultipartUpload(
            handler.abort_multipart_upload(AbortMultipartUploadRequest { object: mk(), upload_id: get(&q, "uploadId").unwrap_or_default() }).await?,
        ),
        Method::DELETE if has(&q, "tagging") => S3Response::DeleteObjectTagging(
            handler.delete_object_tagging(DeleteObjectTaggingRequest { object: mk() }).await?,
        ),
        Method::DELETE => S3Response::DeleteObject(
            handler.delete_object(DeleteObjectRequest { object: mk(), version_id: get(&q, "versionId") }).await?,
        ),
        _ => return Err(HandlerError::method_not_allowed("unsupported object API")),
    };
    Ok(resp)
}
