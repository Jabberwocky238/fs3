use async_trait::async_trait;

use crate::types::s3::core::ObjectReadOptions;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{S3EngineError, S3MultipartEngine, S3ObjectEngine};

use super::utils::*;

#[async_trait]
pub trait ObjectS3Handler<E: S3EngineError + From<S3HandlerBridgeError>>: Send + Sync {
    type Engine: S3ObjectEngine<E> + S3MultipartEngine<E> + Send + Sync;

    fn engine(&self) -> &Self::Engine;

    async fn head_object(&self, req: HeadObjectRequest) -> Result<HeadObjectResponse, E> {
        let obj = self
            .engine()
            .head_object(
                &req.object.bucket,
                &req.object.object,
                ObjectReadOptions::from(&req),
            )
            .await
            ?;
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
        let obj = self
            .engine()
            .head_object(&bucket, &object, opts)
            .await
            ?;
        Ok(GetObjectAttributesResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn copy_object_part(
        &self,
        req: CopyObjectPartRequest,
    ) -> Result<CopyObjectPartResponse, E> {
        let (src_bucket, src_key) = split_copy_source(&req.copy_source)
            .ok_or_else(|| S3HandlerBridgeError::InvalidRequest("missing/invalid x-amz-copy-source".to_string()))
            ?;
        let part = self
            .engine()
            .copy_object_part(
                &src_bucket,
                &src_key,
                &req.object.bucket,
                &req.object.object,
                &req.multipart.upload_id,
                req.multipart.part_number.ok_or_else(|| S3HandlerBridgeError::InvalidRequest("missing partNumber".to_string()))?,
            )
            .await
            ?;
        Ok(CopyObjectPartResponse {
            part: Some(MultipartPartInfo {
                part_number: part.part_number,
                etag: Some(part.etag),
                size: part.size,
            }),
            ..Default::default()
        })
    }

    async fn put_object_part(
        &self,
        req: PutObjectPartRequest,
    ) -> Result<PutObjectPartResponse, E> {
        let part = self
            .engine()
            .put_object_part(
                &req.object.bucket,
                &req.object.object,
                &req.multipart.upload_id,
                req.multipart.part_number.ok_or_else(|| {
                    S3HandlerBridgeError::InvalidRequest("missing partNumber".to_string())
                })?,
                req.body,
            )
            .await
            ?;
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
            .await
            ?;
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
        let completed = parse_completed_parts(&req.xml);
        let obj = self
            .engine()
            .complete_multipart_upload(
                &req.object.bucket,
                &req.object.object,
                &req.upload_id,
                completed,
            )
            .await
            ?;
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
                to_write_opt(None),
            )
            .await
            ?;
        Ok(NewMultipartUploadResponse {
            upload_id: Some(mp.upload_id),
            ..Default::default()
        })
    }

    async fn abort_multipart_upload(
        &self,
        req: AbortMultipartUploadRequest,
    ) -> Result<AbortMultipartUploadResponse, E> {
        self.engine()
            .abort_multipart_upload(&req.object.bucket, &req.object.object, &req.upload_id)
            .await
            ?;
        Ok(Default::default())
    }

    async fn get_object_acl(&self, _req: GetObjectAclRequest) -> Result<GetObjectAclResponse, E> {
        Ok(Default::default())
    }
    async fn put_object_acl(&self, _req: PutObjectAclRequest) -> Result<PutObjectAclResponse, E> {
        Ok(Default::default())
    }

    async fn get_object_tagging(
        &self,
        req: GetObjectTaggingRequest,
    ) -> Result<GetObjectTaggingResponse, E> {
        let tags = self
            .engine()
            .get_object_tagging(&req.object.bucket, &req.object.object)
            .await
            ?;
        let xml = if tags.is_empty() {
            None
        } else {
            Some(
                tags.into_iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join("&"),
            )
        };
        Ok(GetObjectTaggingResponse { xml, ..Default::default() })
    }

    async fn put_object_tagging(
        &self,
        req: PutObjectTaggingRequest,
    ) -> Result<PutObjectTaggingResponse, E> {
        let mut tags = std::collections::HashMap::new();
        if !req.xml.is_empty() {
            tags.insert("raw".to_string(), req.xml);
        }
        self.engine()
            .put_object_tagging(&req.object.bucket, &req.object.object, tags)
            .await
            ?;
        Ok(Default::default())
    }

    async fn delete_object_tagging(
        &self,
        req: DeleteObjectTaggingRequest,
    ) -> Result<DeleteObjectTaggingResponse, E> {
        self.engine()
            .delete_object_tagging(&req.object.bucket, &req.object.object)
            .await
            ?;
        Ok(Default::default())
    }

    async fn select_object_content(
        &self,
        _req: SelectObjectContentRequest,
    ) -> Result<SelectObjectContentResponse, E> {
        unsupported("SelectObjectContent")
    }

    async fn get_object_retention(
        &self,
        req: GetObjectRetentionRequest,
    ) -> Result<GetObjectRetentionResponse, E> {
        let ret = self
            .engine()
            .get_object_retention(&req.object.bucket, &req.object.object)
            .await
            ?;
        Ok(GetObjectRetentionResponse {
            xml: ret.map(|r| format!("{:?}:{}", r.mode, r.retain_until.to_rfc3339())),
            ..Default::default()
        })
    }

    async fn get_object_legal_hold(
        &self,
        req: GetObjectLegalHoldRequest,
    ) -> Result<GetObjectLegalHoldResponse, E> {
        let hold = self
            .engine()
            .get_object_legal_hold(&req.object.bucket, &req.object.object)
            .await
            ?;
        Ok(GetObjectLegalHoldResponse {
            xml: hold.map(|h| if h.enabled { "ON".to_string() } else { "OFF".to_string() }),
            ..Default::default()
        })
    }

    async fn get_object_lambda(&self, req: GetObjectLambdaRequest) -> Result<GetObjectLambdaResponse, E> {
        let (_obj, stream) = self
            .engine()
            .get_object(
                &req.object.bucket,
                &req.object.object,
                ObjectReadOptions::from(&req),
            )
            .await?;
        use futures::TryStreamExt;
        let chunks: Vec<bytes::Bytes> = stream.try_collect().await
            .map_err(|e| S3HandlerBridgeError::InvalidRequest(format!("stream error: {e}")))?;
        let mut buf = Vec::new();
        for c in chunks { buf.extend_from_slice(&c); }
        Ok(GetObjectLambdaResponse { body: buf, ..Default::default() })
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
        Ok(GetObjectResponse {
            meta: ResponseMeta { etag: Some(obj.etag.clone()), ..Default::default() },
            body: stream,
        })
    }

    async fn copy_object(&self, req: CopyObjectRequest) -> Result<CopyObjectResponse, E> {
        let (src_bucket, src_key) = split_copy_source(&req.copy_source)
            .ok_or_else(|| S3HandlerBridgeError::InvalidRequest("missing/invalid x-amz-copy-source".to_string()))
            ?;
        let obj = self
            .engine()
            .copy_object(
                &src_bucket,
                &src_key,
                &req.object.bucket,
                &req.object.object,
                to_write_opt(None),
            )
            .await
            ?;
        Ok(CopyObjectResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn put_object_retention(
        &self,
        req: PutObjectRetentionRequest,
    ) -> Result<PutObjectRetentionResponse, E> {
        if req.xml.is_empty() {
            return unsupported("PutObjectRetention");
        }
        unsupported("PutObjectRetention(xml parsing not implemented)")
    }

    async fn put_object_legal_hold(
        &self,
        req: PutObjectLegalHoldRequest,
    ) -> Result<PutObjectLegalHoldResponse, E> {
        if req.xml.is_empty() {
            return unsupported("PutObjectLegalHold");
        }
        unsupported("PutObjectLegalHold(xml parsing not implemented)")
    }

    async fn put_object_extract(
        &self,
        req: PutObjectExtractRequest,
    ) -> Result<PutObjectExtractResponse, E> {
        let stream: crate::types::s3::core::BoxByteStream = Box::pin(futures::stream::once(async { Ok(bytes::Bytes::from(req.body)) }));
        let _ = self
            .engine()
            .put_object(&req.object.bucket, &req.object.object, stream, to_write_opt(None))
            .await?;
        Ok(PutObjectExtractResponse { extracted_count: 1, ..Default::default() })
    }

    async fn append_object_rejected(
        &self,
        _req: AppendObjectRejectedRequest,
    ) -> Result<AppendObjectRejectedResponse, E> {
        unsupported("AppendObjectRejected")
    }

    async fn put_object(&self, req: PutObjectRequest) -> Result<PutObjectResponse, E> {
        let opt = to_write_opt(req.content_type);
        let obj = self
            .engine()
            .put_object(&req.object.bucket, &req.object.object, req.body, opt)
            .await
            ?;
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
            .await
            ?;
        Ok(Default::default())
    }

    async fn post_restore_object(
        &self,
        _req: PostRestoreObjectRequest,
    ) -> Result<PostRestoreObjectResponse, E> {
        unsupported("PostRestoreObject")
    }
}