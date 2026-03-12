use crate::types::FS3Error;
use crate::types::s3::core::*;
use crate::types::s3::policy::S3Action;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::StdError;
use crate::types::traits::s3_engine::{S3MultipartEngine, S3ObjectEngine};
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use async_trait::async_trait;
use axum::http::{HeaderMap, HeaderName, HeaderValue};

use super::request_validation::{
    DecodedAwsChunkedStream, ParsedServerSideEncryption, RequestValidationPlan,
    ValidatingRequestStream, decode_content_md5, parse_aws_chunked_upload,
    parse_checksum_headers, validate_sse_headers,
};
use super::utils::*;

#[async_trait]
pub trait ObjectS3Handler<E>: Send + Sync
where
    E: StdError + From<FS3Error> + From<S3HandlerBridgeError> + From<std::io::Error>,
{
    type Engine: S3ObjectEngine<E> + S3MultipartEngine<E> + Send + Sync;
    type Policy: S3PolicyEngine<E>;

    fn engine(&self) -> &Self::Engine;
    fn policy(&self) -> &Self::Policy;

    async fn head_object(&self, req: HeadObjectRequest) -> Result<HeadObjectResponse, E> {

        let obj = self
            .engine()
            .head_object(
                &req.object.bucket,
                &req.object.object,
                ObjectReadOptions::from(&req),
            )
            .await?;
        if let Some(ref if_match) = req.if_match {
            if &obj.etag != if_match {
                return Err(S3HandlerBridgeError::PreconditionFailed.into());
            }
        }
        if let Some(ref if_none_match) = req.if_none_match {
            if &obj.etag == if_none_match {
                return Err(S3HandlerBridgeError::NotModified.into());
            }
        }
        let mut headers = std::collections::HashMap::new();
        headers.insert("content-length".to_string(), obj.size.to_string());
        if let Some(ct) = &obj.content_type {
            headers.insert("content-type".to_string(), ct.clone());
        }
        headers.insert("etag".to_string(), obj.etag.clone());
        headers.insert("last-modified".to_string(), obj.last_modified.to_rfc2822());
        Ok(HeadObjectResponse {
            object: Some(to_resp_object(&obj)),
            headers,
            ..Default::default()
        })
    }

    async fn get_object_attributes(
        &self,
        req: GetObjectAttributesRequest,
    ) -> Result<GetObjectAttributesResponse, E> {

        let bucket = req.object.bucket.clone();
        let object = req.object.object.clone();
        let opts: ObjectReadOptions = req.into();
        let obj = self.engine().head_object(&bucket, &object, opts).await?;
        Ok(GetObjectAttributesResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn copy_object_part(
        &self,
        req: CopyObjectPartRequest,
    ) -> Result<CopyObjectPartResponse, E> {
        let (src_bucket, src_key) = split_copy_source(&req.copy_source).ok_or_else(|| {
            S3HandlerBridgeError::InvalidRequest("missing/invalid x-amz-copy-source".to_string())
        })?;
        let part = self
            .engine()
            .copy_object_part(
                &src_bucket,
                &src_key,
                &req.object.bucket,
                &req.object.object,
                &req.multipart.upload_id,
                req.multipart.part_number.ok_or_else(|| {
                    S3HandlerBridgeError::InvalidRequest("missing partNumber".to_string())
                })?,
            )
            .await?;
        Ok(CopyObjectPartResponse {
            part: Some(MultipartPartInfo {
                part_number: part.part_number,
                etag: Some(part.etag),
                size: part.size,
            }),
            ..Default::default()
        })
    }

    async fn put_object_part(&self, req: PutObjectPartRequest) -> Result<PutObjectPartResponse, E> {
        let validation_headers = request_headers_for_part(&req)?;
        let chunked = parse_aws_chunked_upload(&validation_headers)?;
        let checksum = parse_checksum_headers(&validation_headers)?;
        let content_md5 = req
            .content_md5
            .as_deref()
            .map(decode_content_md5)
            .transpose()?;
        let body = if let Some(chunked) = chunked.as_ref() {
            let (stream, _result) = DecodedAwsChunkedStream::new(req.body, chunked)?;
            stream.into_boxed_stream()
        } else {
            req.body
        };
        let body = if content_md5.is_some() || checksum.is_some() {
            ValidatingRequestStream::new(
                body,
                RequestValidationPlan {
                    content_md5,
                    checksum,
                },
            )
            .into_boxed_stream()
        } else {
            body
        };
        let part = self
            .engine()
            .put_object_part(
                &req.object.bucket,
                &req.object.object,
                &req.multipart.upload_id,
                req.multipart.part_number.ok_or_else(|| {
                    S3HandlerBridgeError::InvalidRequest("missing partNumber".to_string())
                })?,
                body,
            )
            .await?;
        Ok(PutObjectPartResponse {
            part: Some(MultipartPartInfo {
                part_number: part.part_number,
                etag: Some(part.etag),
                size: part.size,
            }),
            ..Default::default()
        })
    }

    async fn list_object_parts(
        &self,
        req: ListObjectPartsRequest,
    ) -> Result<ListObjectPartsResponse, E> {

        let parts = self
            .engine()
            .list_object_parts(&req.object.bucket, &req.object.object, &req.upload_id)
            .await?;
        Ok(ListObjectPartsResponse {
            upload_id: Some(req.upload_id),
            parts: parts
                .into_iter()
                .map(|p| MultipartPartInfo {
                    part_number: p.part_number,
                    etag: Some(p.etag),
                    size: p.size,
                })
                .collect(),
            ..Default::default()
        })
    }

    async fn complete_multipart_upload(
        &self,
        req: CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResponse, E> {
        let obj = self
            .engine()
            .complete_multipart_upload(
                &req.object.bucket,
                &req.object.object,
                &req.upload_id,
                req.completed,
            )
            .await?;
        Ok(CompleteMultipartUploadResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn new_multipart_upload(
        &self,
        req: NewMultipartUploadRequest,
    ) -> Result<NewMultipartUploadResponse, E> {
        let mp = self
            .engine()
            .new_multipart_upload(
                &req.object.bucket,
                &req.object.object,
                to_write_opt(None, 0, Default::default()),
            )
            .await?;
        Ok(NewMultipartUploadResponse {
            upload_id: Some(mp.upload_id),
            bucket: Some(req.object.bucket),
            key: Some(req.object.object),
            ..Default::default()
        })
    }

    async fn abort_multipart_upload(
        &self,
        req: AbortMultipartUploadRequest,
    ) -> Result<AbortMultipartUploadResponse, E> {
        self.engine()
            .abort_multipart_upload(&req.object.bucket, &req.object.object, &req.upload_id)
            .await?;
        Ok(AbortMultipartUploadResponse {
            upload_id: Some(req.upload_id),
            ..Default::default()
        })
    }

    async fn get_object_acl(&self, _req: GetObjectAclRequest) -> Result<GetObjectAclResponse, E> {
        Ok(Default::default())
    }

    async fn put_object_acl(&self, _req: PutObjectAclRequest) -> Result<PutObjectAclResponse, E> {
        Ok(Default::default())
    }

    async fn select_object_content(
        &self,
        req: SelectObjectContentRequest,
    ) -> Result<SelectObjectContentResponse, E> {

        unimplemented!()
    }

    async fn get_object_lambda(
        &self,
        req: GetObjectLambdaRequest,
    ) -> Result<GetObjectLambdaResponse, E> {

        let (_obj, stream) = self
            .engine()
            .get_object(
                &req.object.bucket,
                &req.object.object,
                ObjectReadOptions::from(&req),
            )
            .await?;
        use futures::TryStreamExt;
        let chunks: Vec<bytes::Bytes> = stream.try_collect().await?;
        let mut buf = Vec::new();
        for c in chunks {
            buf.extend_from_slice(&c);
        }
        Ok(GetObjectLambdaResponse {
            body: buf,
            ..Default::default()
        })
    }

    async fn get_object(&self, req: GetObjectRequest) -> Result<GetObjectResponse, E> {

        let (obj, stream) = self
            .engine()
            .get_object(
                &req.object.bucket,
                &req.object.object,
                ObjectReadOptions::from(&req),
            )
            .await?;
        if let Some(ref if_match) = req.if_match {
            if &obj.etag != if_match {
                return Err(S3HandlerBridgeError::PreconditionFailed.into());
            }
        }
        if let Some(ref if_none_match) = req.if_none_match {
            if &obj.etag == if_none_match {
                return Err(S3HandlerBridgeError::NotModified.into());
            }
        }
        Ok(GetObjectResponse {
            meta: ResponseMeta {
                etag: Some(obj.etag.clone()),
                ..Default::default()
            },
            size: Some(obj.size),
            body: stream,
        })
    }

    async fn copy_object(&self, req: CopyObjectRequest) -> Result<CopyObjectResponse, E> {
        let src = parse_copy_source(&req.copy_source).ok_or_else(|| {
            S3HandlerBridgeError::InvalidRequest("missing/invalid x-amz-copy-source".to_string())
        })?;
        let mut write_options = copy_to_write_opt(&req);
        if write_options.copy_source_version_id.is_none() {
            write_options.copy_source_version_id = src.version_id.clone();
        }
        let obj = self
            .engine()
            .copy_object(
                &src.bucket,
                &src.key,
                &req.object.bucket,
                &req.object.object,
                write_options,
            )
            .await?;
        Ok(CopyObjectResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn put_object_extract(
        &self,
        req: PutObjectExtractRequest,
    ) -> Result<PutObjectExtractResponse, E> {
        let _ = self
            .engine()
            .put_object(
                &req.object.bucket,
                &req.object.object,
                req.body,
                to_write_opt(None, 0, Default::default()),
            )
            .await?;
        Ok(PutObjectExtractResponse {
            extracted_count: 1,
            ..Default::default()
        })
    }

    async fn append_object_rejected(
        &self,
        _req: AppendObjectRejectedRequest,
    ) -> Result<AppendObjectRejectedResponse, E> {
        unimplemented!()
    }

    async fn put_object(&self, req: PutObjectRequest) -> Result<PutObjectResponse, E> {
        let validation_headers = request_headers_for_put(&req)?;
        let chunked = parse_aws_chunked_upload(&validation_headers)?;
        let checksum = parse_checksum_headers(&validation_headers)?;
        let content_md5 = req
            .content_md5
            .as_deref()
            .map(decode_content_md5)
            .transpose()?;
        let sse = validate_sse_headers(&validation_headers)?;
        let body = if let Some(chunked) = chunked.as_ref() {
            let (stream, _result) = DecodedAwsChunkedStream::new(req.body, chunked)?;
            stream.into_boxed_stream()
        } else {
            req.body
        };
        let body = if content_md5.is_some() || checksum.is_some() {
            ValidatingRequestStream::new(
                body,
                RequestValidationPlan {
                    content_md5,
                    checksum: checksum.clone(),
                },
            )
            .into_boxed_stream()
        } else {
            body
        };
        let mut opt = to_write_opt(
            req.content_type,
            req.content_length.unwrap_or(0),
            req.user_metadata,
        );
        opt.checksum = checksum.map(|checksum| crate::types::s3::core::ObjectChecksum {
            algorithm: checksum.header_name.to_string(),
            value: checksum.base64_value,
        });
        opt.sse_algorithm = sse.map(map_sse_algorithm);
        let obj = self
            .engine()
            .put_object(&req.object.bucket, &req.object.object, body, opt)
            .await?;
        Ok(PutObjectResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn delete_object(&self, req: DeleteObjectRequest) -> Result<DeleteObjectResponse, E> {

        let _ = self
            .engine()
            .delete_object(
                &req.object.bucket,
                &req.object.object,
                to_delete_opt(req.version_id),
            )
            .await?;
        Ok(Default::default())
    }

    async fn post_restore_object(
        &self,
        _req: PostRestoreObjectRequest,
    ) -> Result<PostRestoreObjectResponse, E> {
        unimplemented!()
    }
}

fn map_sse_algorithm(sse: ParsedServerSideEncryption) -> String {
    match sse {
        ParsedServerSideEncryption::S3 { algorithm } => algorithm,
        ParsedServerSideEncryption::Customer(_) => "AES256-C".to_string(),
        ParsedServerSideEncryption::Kms(_) => "aws:kms".to_string(),
    }
}

fn request_headers_for_put(req: &PutObjectRequest) -> Result<HeaderMap, FS3Error> {
    let mut headers = HeaderMap::new();
    insert_opt_header(&mut headers, "content-md5", req.content_md5.as_deref())?;
    insert_opt_header(
        &mut headers,
        "x-amz-checksum-sha256",
        req.checksum_sha256.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-checksum-sha1",
        req.checksum_sha1.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-checksum-crc32",
        req.checksum_crc32.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-checksum-crc32c",
        req.checksum_crc32c.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "content-encoding",
        req.content_encoding.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-content-sha256",
        req.amz_content_sha256.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-decoded-content-length",
        req.decoded_content_length.as_deref(),
    )?;
    insert_opt_header(&mut headers, "x-amz-trailer", req.amz_trailer.as_deref())?;
    insert_opt_header(
        &mut headers,
        "x-amz-server-side-encryption",
        req.sse.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-server-side-encryption-customer-algorithm",
        req.sse_customer_algorithm.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-server-side-encryption-customer-key",
        req.sse_customer_key.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-server-side-encryption-customer-key-md5",
        req.sse_customer_key_md5.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-server-side-encryption-aws-kms-key-id",
        req.sse_kms_key_id.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-server-side-encryption-context",
        req.sse_context.as_deref(),
    )?;
    Ok(headers)
}

fn request_headers_for_part(req: &PutObjectPartRequest) -> Result<HeaderMap, FS3Error> {
    let mut headers = HeaderMap::new();
    insert_opt_header(&mut headers, "content-md5", req.content_md5.as_deref())?;
    insert_opt_header(
        &mut headers,
        "x-amz-checksum-sha256",
        req.checksum.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "content-encoding",
        req.content_encoding.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-content-sha256",
        req.amz_content_sha256.as_deref(),
    )?;
    insert_opt_header(
        &mut headers,
        "x-amz-decoded-content-length",
        req.decoded_content_length.as_deref(),
    )?;
    insert_opt_header(&mut headers, "x-amz-trailer", req.amz_trailer.as_deref())?;
    Ok(headers)
}

fn insert_opt_header(
    headers: &mut HeaderMap,
    name: &'static str,
    value: Option<&str>,
) -> Result<(), FS3Error> {
    let Some(value) = value else {
        return Ok(());
    };
    let header_name = HeaderName::from_static(name);
    let header_value =
        HeaderValue::from_str(value).map_err(|_| FS3Error::bad_request("InvalidRequest"))?;
    headers.insert(header_name, header_value);
    Ok(())
}

