use async_trait::async_trait;

use crate::types::s3::core::{
    ObjectReadOptions,
};
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{
    S3MultipartEngine, S3ObjectEngine,
};

use super::utils::*;

#[async_trait]
pub trait ObjectS3Handler
where
    <Self::Engine as S3ObjectEngine>::Error: Into<Self::Error>,
    <Self::Engine as S3MultipartEngine>::Error: Into<Self::Error>,
    Self::Error: From<S3HandlerBridgeError>,
{
    type Engine: S3ObjectEngine + S3MultipartEngine;
    type Error: Send + Sync + 'static;

    fn engine(&self) -> &Self::Engine;

    async fn head_object(&self, req: HeadObjectRequest) -> Result<HeadObjectResponse, Self::Error> {
        let obj = self
            .engine()
            .head_object(
                &req.object.bucket,
                &req.object.object,
                ObjectReadOptions::from(&req),
            )
            .await
            .map_err(Into::into)?;
        Ok(HeadObjectResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn get_object_attributes(
        &self,
        req: GetObjectAttributesRequest,
    ) -> Result<GetObjectAttributesResponse, Self::Error> {
        let bucket = req.object.bucket.clone();
        let object = req.object.object.clone();
        let opts: ObjectReadOptions = req.into();
        let obj = self
            .engine()
            .head_object(&bucket, &object, opts)
            .await
            .map_err(Into::into)?;
        Ok(GetObjectAttributesResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn copy_object_part(
        &self,
        req: CopyObjectPartRequest,
    ) -> Result<CopyObjectPartResponse, Self::Error> {
        let (src_bucket, src_key) = split_copy_source(&req.copy_source)
            .ok_or_else(|| S3HandlerBridgeError::InvalidRequest("missing/invalid x-amz-copy-source".to_string()))
            .map_err(Into::into)?;
        let part = self
            .engine()
            .copy_object_part(
                &src_bucket,
                &src_key,
                &req.object.bucket,
                &req.object.object,
                &req.multipart.upload_id,
                req.multipart.part_number.ok_or_else(|| S3HandlerBridgeError::InvalidRequest("missing partNumber".to_string())).map_err(Into::into)?,
            )
            .await
            .map_err(Into::into)?;
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
    ) -> Result<PutObjectPartResponse, Self::Error> {
        let part = self
            .engine()
            .put_object_part(
                &req.object.bucket,
                &req.object.object,
                &req.multipart.upload_id,
                req.multipart.part_number.ok_or_else(|| {
                    S3HandlerBridgeError::InvalidRequest("missing partNumber".to_string())
                }).map_err(Into::into)?,
                req.body,
            )
            .await
            .map_err(Into::into)?;
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
    ) -> Result<ListObjectPartsResponse, Self::Error> {
        let parts = self
            .engine()
            .list_object_parts(&req.object.bucket, &req.object.object, &req.upload_id)
            .await
            .map_err(Into::into)?;
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
    ) -> Result<CompleteMultipartUploadResponse, Self::Error> {
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
            .map_err(Into::into)?;
        Ok(CompleteMultipartUploadResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn new_multipart_upload(
        &self,
        req: NewMultipartUploadRequest,
    ) -> Result<NewMultipartUploadResponse, Self::Error> {
        let mp = self
            .engine()
            .new_multipart_upload(
                &req.object.bucket,
                &req.object.object,
                to_write_opt(None),
            )
            .await
            .map_err(Into::into)?;
        Ok(NewMultipartUploadResponse {
            upload_id: Some(mp.upload_id),
            ..Default::default()
        })
    }

    async fn abort_multipart_upload(
        &self,
        req: AbortMultipartUploadRequest,
    ) -> Result<AbortMultipartUploadResponse, Self::Error> {
        self.engine()
            .abort_multipart_upload(&req.object.bucket, &req.object.object, &req.upload_id)
            .await
            .map_err(Into::into)?;
        Ok(Default::default())
    }

    async fn get_object_acl(&self, _req: GetObjectAclRequest) -> Result<GetObjectAclResponse, Self::Error> {
        Ok(Default::default())
    }
    async fn put_object_acl(&self, _req: PutObjectAclRequest) -> Result<PutObjectAclResponse, Self::Error> {
        Ok(Default::default())
    }

    async fn get_object_tagging(
        &self,
        req: GetObjectTaggingRequest,
    ) -> Result<GetObjectTaggingResponse, Self::Error> {
        let tags = self
            .engine()
            .get_object_tagging(&req.object.bucket, &req.object.object)
            .await
            .map_err(Into::into)?;
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
    ) -> Result<PutObjectTaggingResponse, Self::Error> {
        let mut tags = std::collections::HashMap::new();
        if !req.xml.is_empty() {
            tags.insert("raw".to_string(), req.xml);
        }
        self.engine()
            .put_object_tagging(&req.object.bucket, &req.object.object, tags)
            .await
            .map_err(Into::into)?;
        Ok(Default::default())
    }

    async fn delete_object_tagging(
        &self,
        req: DeleteObjectTaggingRequest,
    ) -> Result<DeleteObjectTaggingResponse, Self::Error> {
        self.engine()
            .delete_object_tagging(&req.object.bucket, &req.object.object)
            .await
            .map_err(Into::into)?;
        Ok(Default::default())
    }

    async fn select_object_content(
        &self,
        _req: SelectObjectContentRequest,
    ) -> Result<SelectObjectContentResponse, Self::Error> {
        unsupported("SelectObjectContent")
    }

    async fn get_object_retention(
        &self,
        req: GetObjectRetentionRequest,
    ) -> Result<GetObjectRetentionResponse, Self::Error> {
        let ret = self
            .engine()
            .get_object_retention(&req.object.bucket, &req.object.object)
            .await
            .map_err(Into::into)?;
        Ok(GetObjectRetentionResponse {
            xml: ret.map(|r| format!("{:?}:{}", r.mode, r.retain_until.to_rfc3339())),
            ..Default::default()
        })
    }

    async fn get_object_legal_hold(
        &self,
        req: GetObjectLegalHoldRequest,
    ) -> Result<GetObjectLegalHoldResponse, Self::Error> {
        let hold = self
            .engine()
            .get_object_legal_hold(&req.object.bucket, &req.object.object)
            .await
            .map_err(Into::into)?;
        Ok(GetObjectLegalHoldResponse {
            xml: hold.map(|h| if h.enabled { "ON".to_string() } else { "OFF".to_string() }),
            ..Default::default()
        })
    }

    async fn get_object_lambda(&self, req: GetObjectLambdaRequest) -> Result<GetObjectLambdaResponse, Self::Error> {
        let got = self
            .engine()
            .get_object(
                &req.object.bucket,
                &req.object.object,
                ObjectReadOptions::from(&req),
            )
            .await
            .map_err(Into::into)?;
        Ok(GetObjectLambdaResponse {
            body: got.1,
            ..Default::default()
        })
    }

    async fn get_object(&self, req: GetObjectRequest) -> Result<GetObjectResponse, Self::Error> {
        let got = self
            .engine()
            .get_object(
                &req.object.bucket,
                &req.object.object,
                ObjectReadOptions::from(&req),
            )
            .await
            .map_err(Into::into)?;
        Ok(GetObjectResponse {
            body: got.1,
            ..Default::default()
        })
    }

    async fn copy_object(&self, req: CopyObjectRequest) -> Result<CopyObjectResponse, Self::Error> {
        let (src_bucket, src_key) = split_copy_source(&req.copy_source)
            .ok_or_else(|| S3HandlerBridgeError::InvalidRequest("missing/invalid x-amz-copy-source".to_string()))
            .map_err(Into::into)?;
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
            .map_err(Into::into)?;
        Ok(CopyObjectResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn put_object_retention(
        &self,
        req: PutObjectRetentionRequest,
    ) -> Result<PutObjectRetentionResponse, Self::Error> {
        if req.xml.is_empty() {
            return unsupported("PutObjectRetention");
        }
        unsupported("PutObjectRetention(xml parsing not implemented)")
    }

    async fn put_object_legal_hold(
        &self,
        req: PutObjectLegalHoldRequest,
    ) -> Result<PutObjectLegalHoldResponse, Self::Error> {
        if req.xml.is_empty() {
            return unsupported("PutObjectLegalHold");
        }
        unsupported("PutObjectLegalHold(xml parsing not implemented)")
    }

    async fn put_object_extract(
        &self,
        req: PutObjectExtractRequest,
    ) -> Result<PutObjectExtractResponse, Self::Error> {
        let _ = self
            .engine()
            .put_object(
                &req.object.bucket,
                &req.object.object,
                req.body,
                to_write_opt(None),
            )
            .await
            .map_err(Into::into)?;
        Ok(PutObjectExtractResponse { extracted_count: 1, ..Default::default() })
    }

    async fn append_object_rejected(
        &self,
        _req: AppendObjectRejectedRequest,
    ) -> Result<AppendObjectRejectedResponse, Self::Error> {
        unsupported("AppendObjectRejected")
    }

    async fn put_object(&self, req: PutObjectRequest) -> Result<PutObjectResponse, Self::Error> {
        let opt = to_write_opt(req.content_type);
        let obj = self
            .engine()
            .put_object(&req.object.bucket, &req.object.object, req.body, opt)
            .await
            .map_err(Into::into)?;
        Ok(PutObjectResponse {
            object: Some(to_resp_object(&obj)),
            ..Default::default()
        })
    }

    async fn delete_object(&self, req: DeleteObjectRequest) -> Result<DeleteObjectResponse, Self::Error> {
        let _ = self
            .engine()
            .delete_object(
                &req.object.bucket,
                &req.object.object,
                to_delete_opt(req.version_id),
            )
            .await
            .map_err(Into::into)?;
        Ok(Default::default())
    }

    async fn post_restore_object(
        &self,
        _req: PostRestoreObjectRequest,
    ) -> Result<PostRestoreObjectResponse, Self::Error> {
        unsupported("PostRestoreObject")
    }
}