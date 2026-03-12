use std::collections::HashMap;

use axum::http::{HeaderMap, Method};

use crate::types::s3::core::{BoxByteStream, ObjectAttribute};
use crate::types::s3::request::*;

pub struct ObjectEntryRef<'a> {
    pub bucket: &'a str,
    pub object_path: &'a str,
    pub method: &'a Method,
    pub headers: &'a HeaderMap,
    pub q: &'a HashMap<String, String>,
    pub body: Option<BoxByteStream>,
}

impl<'a> ObjectEntryRef<'a> {
    fn object_ref(&self) -> ObjectRef {
        ObjectRef {
            bucket: self.bucket.to_string(),
            object: self.object_path.to_string(),
        }
    }

    fn bucket_ref(&self) -> BucketRef {
        BucketRef {
            bucket: self.bucket.to_string(),
        }
    }

    fn q(&self, key: &str) -> Option<String> {
        self.q.get(key).cloned()
    }

    fn q_u32(&self, key: &str) -> Option<u32> {
        self.q.get(key).and_then(|x| x.parse::<u32>().ok())
    }

    fn header(&self, key: &str) -> Option<String> {
        self.headers
            .get(key)
            .and_then(|x| x.to_str().ok())
            .map(ToString::to_string)
    }

    fn content_length(&self) -> Option<u64> {
        self.header("content-length").and_then(|x| x.parse::<u64>().ok())
    }

    fn has_body(&self) -> bool {
        self.content_length().map(|len| len > 0).unwrap_or(self.body.is_some())
    }

    fn parse_attrs(&self) -> Vec<ObjectAttribute> {
        self.q("attributes")
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

    fn multipart_selector(&self) -> MultipartSelector {
        MultipartSelector {
            upload_id: self.q("uploadId").unwrap_or_default(),
            part_number: self.q_u32("partNumber"),
        }
    }

    fn into_body(self) -> BoxByteStream {
        self.body.unwrap_or_else(|| Box::pin(futures::stream::empty()))
    }
}

impl<'a> From<ObjectEntryRef<'a>> for HeadObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: v.object_ref(),
            range: v.header("range"),
            version_id: v.q("versionId"),
            if_match: v.header("if-match"),
            if_none_match: v.header("if-none-match"),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectAttributesRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: v.object_ref(),
            attributes: v.parse_attrs(),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for CopyObjectPartRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: v.object_ref(),
            multipart: v.multipart_selector(),
            copy_source: v.header("x-amz-copy-source").unwrap_or_default(),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectPartRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let checksum = v.header("x-amz-checksum-sha256");
        let content_md5 = v.header("content-md5");
        let content_encoding = v.header("content-encoding");
        let amz_content_sha256 = v.header("x-amz-content-sha256");
        let decoded_content_length = v.header("x-amz-decoded-content-length");
        let amz_trailer = v.header("x-amz-trailer");
        let object = v.object_ref();
        let multipart = v.multipart_selector();
        let body = v.into_body();
        Self {
            object,
            multipart,
            body,
            checksum,
            content_md5,
            content_encoding,
            amz_content_sha256,
            decoded_content_length,
            amz_trailer,
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for ListObjectPartsRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: v.object_ref(),
            upload_id: v.q("uploadId").unwrap_or_default(),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for CompleteMultipartUploadRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let object = v.object_ref();
        let upload_id = v.q("uploadId").unwrap_or_default();
        let body = v.into_body();
        Self { object, upload_id, body }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for NewMultipartUploadRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: v.object_ref() }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for AbortMultipartUploadRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: v.object_ref(),
            upload_id: v.q("uploadId").unwrap_or_default(),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectAclRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: v.object_ref() }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectAclRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let object = v.object_ref();
        let body = if v.has_body() { Some(v.into_body()) } else { None };
        Self { object, body }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectTaggingRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: v.object_ref() }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectTaggingRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let object = v.object_ref();
        let body = v.into_body();
        Self { object, body }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for DeleteObjectTaggingRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: v.object_ref() }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for SelectObjectContentRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let object = v.object_ref();
        let select_type = v.q("select-type")
            .and_then(|x| x.parse::<u8>().ok())
            .unwrap_or(2);
        let body = v.into_body();
        Self {
            object,
            select_type,
            body,
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectRetentionRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { bucket: v.bucket_ref(), object: v.object_ref() }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectLegalHoldRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { bucket: v.bucket_ref(), object: v.object_ref() }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectLambdaRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: v.object_ref(),
            lambda_arn: v.q("lambdaArn").unwrap_or_default(),
            range: v.header("range"),
            version_id: v.q("versionId"),
            if_match: v.header("if-match"),
            if_none_match: v.header("if-none-match"),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for GetObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: v.object_ref(),
            range: v.header("range"),
            version_id: v.q("versionId"),
            if_match: v.header("if-match"),
            if_none_match: v.header("if-none-match"),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for CopyObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let mut user_metadata = std::collections::HashMap::new();
        for (key, value) in v.headers.iter() {
            if let Some(meta_key) = key.as_str().strip_prefix("x-amz-meta-") {
                if let Ok(meta_value) = value.to_str() {
                    user_metadata.insert(meta_key.to_string(), meta_value.to_string());
                }
            }
        }
        Self {
            object: v.object_ref(),
            copy_source: v.header("x-amz-copy-source").unwrap_or_default(),
            copy_source_version_id: None,
            metadata_directive: v.header("x-amz-metadata-directive"),
            tagging_directive: v.header("x-amz-tagging-directive"),
            content_type: v.header("content-type"),
            content_encoding: v.header("content-encoding"),
            storage_class: v.header("x-amz-storage-class"),
            user_metadata,
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectRetentionRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let bucket = v.bucket_ref();
        let object = v.object_ref();
        let body = v.into_body();
        Self { bucket, object, body }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectLegalHoldRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let bucket = v.bucket_ref();
        let object = v.object_ref();
        let body = v.into_body();
        Self { bucket, object, body }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectExtractRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let object = v.object_ref();
        let body = v.into_body();
        Self { object, body }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for AppendObjectRejectedRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let object = v.object_ref();
        let write_offset_bytes = v.header("x-amz-write-offset-bytes").unwrap_or_default();
        let body = v.into_body();
        Self {
            object,
            write_offset_bytes,
            body,
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PutObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let object = v.object_ref();
        let content_type = v.header("content-type");
        let content_md5 = v.header("content-md5");
        let checksum_sha256 = v.header("x-amz-checksum-sha256");
        let checksum_sha1 = v.header("x-amz-checksum-sha1");
        let checksum_crc32 = v.header("x-amz-checksum-crc32");
        let checksum_crc32c = v.header("x-amz-checksum-crc32c");
        let content_length = v.content_length();
        let content_encoding = v.header("content-encoding");
        let amz_content_sha256 = v.header("x-amz-content-sha256");
        let decoded_content_length = v.header("x-amz-decoded-content-length");
        let amz_trailer = v.header("x-amz-trailer");
        let sse = v.header("x-amz-server-side-encryption");
        let sse_customer_algorithm = v.header("x-amz-server-side-encryption-customer-algorithm");
        let sse_customer_key = v.header("x-amz-server-side-encryption-customer-key");
        let sse_customer_key_md5 = v.header("x-amz-server-side-encryption-customer-key-md5");
        let sse_kms_key_id = v.header("x-amz-server-side-encryption-aws-kms-key-id");
        let sse_context = v.header("x-amz-server-side-encryption-context");
        let user_metadata = Default::default();
        let body = v.into_body();
        Self {
            object,
            body,
            content_type,
            content_md5,
            checksum_sha256,
            checksum_sha1,
            checksum_crc32,
            checksum_crc32c,
            content_length,
            content_encoding,
            amz_content_sha256,
            decoded_content_length,
            amz_trailer,
            sse,
            sse_customer_algorithm,
            sse_customer_key,
            sse_customer_key_md5,
            sse_kms_key_id,
            sse_context,
            user_metadata,
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for DeleteObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: v.object_ref(),
            version_id: v.q("versionId"),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for PostRestoreObjectRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        let object = v.object_ref();
        let body = v.into_body();
        Self { object, body }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for RejectedObjectTorrentRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self {
            object: v.object_ref(),
            method: v.method.to_string(),
        }
    }
}

impl<'a> From<ObjectEntryRef<'a>> for RejectedObjectAclDeleteRequest {
    fn from(v: ObjectEntryRef<'a>) -> Self {
        Self { object: v.object_ref() }
    }
}

