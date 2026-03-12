use crate::types::FS3Error;
use crate::types::s3::core::*;
use crate::types::s3::policy::S3Action;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{S3MultipartEngine, S3ObjectEngine};
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use async_trait::async_trait;

use super::utils::*;

#[async_trait]
pub trait ObjectS3Handler: Send + Sync {
    type Engine: S3ObjectEngine + S3MultipartEngine + Send + Sync;
    type Policy: S3PolicyEngine;

    fn engine(&self) -> &Self::Engine;
    fn policy(&self) -> &Self::Policy;

    async fn head_object(&self, req: HeadObjectRequest) -> Result<HeadObjectResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::HeadObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
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
    ) -> Result<GetObjectAttributesResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::GetObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
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
    ) -> Result<CopyObjectPartResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::PutObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
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

    async fn put_object_part(
        &self,
        req: PutObjectPartRequest,
    ) -> Result<PutObjectPartResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::PutObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
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
    ) -> Result<ListObjectPartsResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::ListMultipartUploadParts,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
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
    ) -> Result<CompleteMultipartUploadResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::PutObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
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
    ) -> Result<NewMultipartUploadResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::PutObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
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
    ) -> Result<AbortMultipartUploadResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::AbortMultipartUpload,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
        self.engine()
            .abort_multipart_upload(&req.object.bucket, &req.object.object, &req.upload_id)
            .await?;
        Ok(AbortMultipartUploadResponse {
            upload_id: Some(req.upload_id),
            ..Default::default()
        })
    }

    async fn get_object_acl(
        &self,
        _req: GetObjectAclRequest,
    ) -> Result<GetObjectAclResponse, FS3Error> {
        Ok(Default::default())
    }

    async fn put_object_acl(
        &self,
        _req: PutObjectAclRequest,
    ) -> Result<PutObjectAclResponse, FS3Error> {
        Ok(Default::default())
    }

    async fn select_object_content(
        &self,
        req: SelectObjectContentRequest,
    ) -> Result<SelectObjectContentResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::SelectObjectContent,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;

        unimplemented!()
    }

    async fn get_object_lambda(
        &self,
        req: GetObjectLambdaRequest,
    ) -> Result<GetObjectLambdaResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::GetObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
        let (_obj, stream) = self
            .engine()
            .get_object(
                &req.object.bucket,
                &req.object.object,
                ObjectReadOptions::from(&req),
            )
            .await?;
        use futures::TryStreamExt;
        let chunks: Vec<bytes::Bytes> = stream
            .try_collect()
            .await?;
        let mut buf = Vec::new();
        for c in chunks {
            buf.extend_from_slice(&c);
        }
        Ok(GetObjectLambdaResponse {
            body: buf,
            ..Default::default()
        })
    }

    async fn get_object(&self, req: GetObjectRequest) -> Result<GetObjectResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::GetObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
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

    async fn copy_object(&self, req: CopyObjectRequest) -> Result<CopyObjectResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::PutObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
        let (src_bucket, src_key) = split_copy_source(&req.copy_source).ok_or_else(|| {
            S3HandlerBridgeError::InvalidRequest("missing/invalid x-amz-copy-source".to_string())
        })?;
        let obj = self
            .engine()
            .copy_object(
                &src_bucket,
                &src_key,
                &req.object.bucket,
                &req.object.object,
                to_write_opt(None, 0, Default::default()),
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
    ) -> Result<PutObjectExtractResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::PutObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
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
    ) -> Result<AppendObjectRejectedResponse, FS3Error> {
        unimplemented!()
    }

    async fn put_object(&self, req: PutObjectRequest) -> Result<PutObjectResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::PutObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;

        let _content_md5 = req.content_md5;
        let opt = to_write_opt(
            req.content_type,
            req.content_length.unwrap_or(0),
            req.user_metadata,
        );
        let obj = self
            .engine()
            .put_object(&req.object.bucket, &req.object.object, req.body, opt)
            .await?;
        Ok(PutObjectResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn delete_object(
        &self,
        req: DeleteObjectRequest,
    ) -> Result<DeleteObjectResponse, FS3Error> {
        check_access(
            self.policy(),
            S3Action::DeleteObject,
            Some(&req.object.bucket),
            Some(&req.object.object),
        )
        .await?;
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
    ) -> Result<PostRestoreObjectResponse, FS3Error> {
        unimplemented!()
    }
}
