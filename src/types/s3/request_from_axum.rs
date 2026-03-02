use std::collections::HashMap;

use axum::http::{HeaderMap, Method};

use crate::types::s3::core::ObjectAttribute;
use crate::types::s3::request::*;

// Zero-copy view of an object-level axum request context.
// All fields are immutable references.
#[derive(Debug, Clone, Copy)]
pub struct ObjectEntryRef<'a> {
    pub bucket: &'a str,
    pub object_path: &'a str,
    pub method: &'a Method,
    pub headers: &'a HeaderMap,
    pub q: &'a HashMap<String, String>,
    pub body: &'a [u8],
}

fn object_ref(v: ObjectEntryRef<'_>) -> ObjectRef {
    ObjectRef {
        bucket: v.bucket.to_string(),
        object: v.object_path.to_string(),
    }
}

fn q(v: ObjectEntryRef<'_>, key: &str) -> Option<String> {
    v.q.get(key).cloned()
}

fn q_u32(v: ObjectEntryRef<'_>, key: &str) -> Option<u32> {
    v.q.get(key).and_then(|x| x.parse::<u32>().ok())
}

fn header(v: ObjectEntryRef<'_>, key: &str) -> Option<String> {
    v.headers
        .get(key)
        .and_then(|x| x.to_str().ok())
        .map(ToString::to_string)
}

fn body_string(v: ObjectEntryRef<'_>) -> String {
    String::from_utf8_lossy(v.body).to_string()
}

fn body_string_opt(v: ObjectEntryRef<'_>) -> Option<String> {
    if v.body.is_empty() {
        None
    } else {
        Some(body_string(v))
    }
}

fn parse_attrs(v: ObjectEntryRef<'_>) -> Vec<ObjectAttribute> {
    q(v, "attributes")
        .map(|s| {
            s.split(',')
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
        .unwrap_or_default()
}

fn multipart_selector(v: ObjectEntryRef<'_>) -> MultipartSelector {
    MultipartSelector {
        upload_id: q(v, "uploadId").unwrap_or_default(),
        part_number: q_u32(v, "partNumber"),
    }
}

impl<'a> From<ObjectEntryRef<'a>> for HeadObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            range: header(v, "range"),
            version_id: q(v, "versionId"),
            if_match: header(v, "if-match"),
            if_none_match: header(v, "if-none-match"),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectAttributesRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            attributes: parse_attrs(v),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for CopyObjectPartRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            multipart: multipart_selector(v),
            copy_source: header(v, "x-amz-copy-source").unwrap_or_default(),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectPartRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let data = bytes::Bytes::copy_from_slice(v.body);
        Self {
            object: object_ref(v),
            multipart: multipart_selector(v),
            body: Box::pin(futures::stream::once(async { Ok(data) })),
            checksum: header(v, "x-amz-checksum-sha256"),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for ListObjectPartsRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            upload_id: q(v, "uploadId").unwrap_or_default(),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for CompleteMultipartUploadRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            upload_id: q(v, "uploadId").unwrap_or_default(),
            xml: body_string(v),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for NewMultipartUploadRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: object_ref(v) }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for AbortMultipartUploadRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            upload_id: q(v, "uploadId").unwrap_or_default(),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectAclRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: object_ref(v) }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectAclRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            xml: body_string_opt(v),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectTaggingRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: object_ref(v) }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectTaggingRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            xml: body_string(v),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for DeleteObjectTaggingRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: object_ref(v) }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for SelectObjectContentRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            select_type: q(v, "select-type")
                .and_then(|x| x.parse::<u8>().ok())
                .unwrap_or(2),
            xml: body_string(v),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectRetentionRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: object_ref(v) }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectLegalHoldRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: object_ref(v) }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectLambdaRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            lambda_arn: q(v, "lambdaArn").unwrap_or_default(),
            range: header(v, "range"),
            version_id: q(v, "versionId"),
            if_match: header(v, "if-match"),
            if_none_match: header(v, "if-none-match"),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            range: header(v, "range"),
            version_id: q(v, "versionId"),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for CopyObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            copy_source: header(v, "x-amz-copy-source").unwrap_or_default(),
            metadata_directive: header(v, "x-amz-metadata-directive"),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectRetentionRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            xml: body_string(v),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectLegalHoldRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            xml: body_string(v),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectExtractRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            body: v.body.to_vec(),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for AppendObjectRejectedRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            write_offset_bytes: header(v, "x-amz-write-offset-bytes").unwrap_or_default(),
            body: v.body.to_vec(),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let data = bytes::Bytes::copy_from_slice(v.body);
        Self {
            object: object_ref(v),
            body: Box::pin(futures::stream::once(async { Ok(data) })),
            content_type: header(v, "content-type"),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for DeleteObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            version_id: q(v, "versionId"),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PostRestoreObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            xml: body_string(v),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for RejectedObjectTorrentRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: object_ref(v),
            method: v.method.to_string(),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for RejectedObjectAclDeleteRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: object_ref(v) }
    }
}
