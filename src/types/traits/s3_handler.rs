use async_trait::async_trait;
use chrono::SecondsFormat;
use thiserror::Error;

use crate::types::s3::core::{
    BucketFeatures, CompleteMultipartInput, DeleteObjectOptions, ListOptions, ObjectReadOptions,
    ObjectWriteOptions, UploadedPart, VersioningState,
};
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{
    S3BucketConfigEngine, S3BucketEngine, S3MultipartEngine, S3ObjectEngine,
};

#[derive(Debug, Error)]
pub enum S3HandlerBridgeError {
    #[error("unsupported by current S3 engine: {0}")]
    Unsupported(&'static str),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

fn unsupported<T, E>(op: &'static str) -> Result<T, E>
where
    E: From<S3HandlerBridgeError>,
{
    Err(S3HandlerBridgeError::Unsupported(op).into())
}

fn to_resp_object(v: &crate::types::s3::core::S3Object) -> ObjectInfo {
    ObjectInfo {
        bucket: v.bucket.clone(),
        key: v.key.clone(),
        size: v.size,
        etag: Some(v.etag.clone()),
        last_modified: Some(v.last_modified.to_rfc3339_opts(SecondsFormat::Secs, true)),
        storage_class: Some(format!("{:?}", v.storage_class)),
    }
}

fn to_read_opt(req: &GetObjectRequest) -> ObjectReadOptions {
    ObjectReadOptions {
        version_id: req.version_id.clone(),
        range: req.range.clone(),
        if_match: None,
        if_none_match: None,
    }
}

fn split_copy_source(s: &str) -> Option<(String, String)> {
    let raw = s.trim_start_matches('/');
    let mut it = raw.splitn(2, '/');
    let b = it.next()?.to_string();
    let k = it.next()?.to_string();
    Some((b, k))
}

fn parse_completed_parts(xml: &str) -> CompleteMultipartInput {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(part_start) = xml[pos..].find("<Part>") {
        let ps = pos + part_start;
        let pe = match xml[ps..].find("</Part>") {
            Some(v) => ps + v,
            None => break,
        };
        let chunk = &xml[ps..pe];
        let pn = extract_tag(chunk, "PartNumber")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or_default();
        let etag = extract_tag(chunk, "ETag").unwrap_or_default();
        out.push(UploadedPart {
            part_number: pn,
            etag,
            size: 0,
        });
        pos = pe + "</Part>".len();
    }
    CompleteMultipartInput { parts: out }
}

fn extract_tag(src: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let s = src.find(&open)? + open.len();
    let e = src[s..].find(&close)? + s;
    Some(
        src[s..e]
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string(),
    )
}

fn parse_delete_keys(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(start) = xml[pos..].find("<Key>") {
        let s = pos + start + "<Key>".len();
        let e = match xml[s..].find("</Key>") {
            Some(v) => s + v,
            None => break,
        };
        out.push(xml[s..e].to_string());
        pos = e + "</Key>".len();
    }
    out
}

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
            .head_object(&req.object.bucket, &req.object.object, ObjectReadOptions::default())
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
        let obj = self
            .engine()
            .head_object(&req.object.bucket, &req.object.object, ObjectReadOptions::default())
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
                req.multipart.part_number.unwrap_or_default(),
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
                ObjectWriteOptions::default(),
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
                ObjectReadOptions::default(),
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
            .get_object(&req.object.bucket, &req.object.object, to_read_opt(&req))
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
                ObjectWriteOptions::default(),
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
                ObjectWriteOptions::default(),
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
        let opt = ObjectWriteOptions {
            content_type: req.content_type,
            versioning: VersioningState::Off,
            ..Default::default()
        };
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
                DeleteObjectOptions {
                    version_id: req.version_id,
                    ..Default::default()
                },
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

#[async_trait]
pub trait BucketS3Handler
where
    <Self::Engine as S3BucketEngine>::Error: Into<Self::Error>,
    <Self::Engine as S3BucketConfigEngine>::Error: Into<Self::Error>,
    <Self::Engine as S3MultipartEngine>::Error: Into<Self::Error>,
    <Self::Engine as S3ObjectEngine>::Error: Into<Self::Error>,
    Self::Error: From<S3HandlerBridgeError>,
{
    type Engine: S3BucketEngine + S3BucketConfigEngine + S3MultipartEngine + S3ObjectEngine;
    type Error: Send + Sync + 'static;
    fn engine(&self) -> &Self::Engine;

    async fn get_bucket_location(&self, req: GetBucketLocationRequest) -> Result<GetBucketLocationResponse, Self::Error> {
        let location = self.engine().get_bucket_location(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketLocationResponse { location: Some(location), ..Default::default() })
    }
    async fn get_bucket_policy(&self, req: GetBucketPolicyRequest) -> Result<GetBucketPolicyResponse, Self::Error> {
        let p = self.engine().get_bucket_policy(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketPolicyResponse { json: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_lifecycle(&self, req: GetBucketLifecycleRequest) -> Result<GetBucketLifecycleResponse, Self::Error> {
        let p = self.engine().get_bucket_lifecycle(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketLifecycleResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_encryption(&self, req: GetBucketEncryptionRequest) -> Result<GetBucketEncryptionResponse, Self::Error> {
        let p = self.engine().get_bucket_encryption(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketEncryptionResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_object_lock_config(&self, req: GetBucketObjectLockConfigRequest) -> Result<GetBucketObjectLockConfigResponse, Self::Error> {
        let p = self.engine().get_bucket_object_lock_config(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketObjectLockConfigResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_replication_config(&self, req: GetBucketReplicationConfigRequest) -> Result<GetBucketReplicationConfigResponse, Self::Error> {
        let p = self.engine().get_bucket_replication(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketReplicationConfigResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_versioning(&self, req: GetBucketVersioningRequest) -> Result<GetBucketVersioningResponse, Self::Error> {
        let p = self.engine().get_bucket_versioning(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketVersioningResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn get_bucket_notification(&self, req: GetBucketNotificationRequest) -> Result<GetBucketNotificationResponse, Self::Error> {
        let p = self.engine().get_bucket_notification(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketNotificationResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn listen_bucket_notification(&self, _req: ListenBucketNotificationRequest) -> Result<ListenBucketNotificationResponse, Self::Error> { unsupported("ListenBucketNotification") }
    async fn reset_bucket_replication_status(&self, _req: ResetBucketReplicationStatusRequest) -> Result<ResetBucketReplicationStatusResponse, Self::Error> { unsupported("ResetBucketReplicationStatus") }
    async fn get_bucket_acl(&self, _req: GetBucketAclRequest) -> Result<GetBucketAclResponse, Self::Error> { Ok(Default::default()) }
    async fn put_bucket_acl(&self, _req: PutBucketAclRequest) -> Result<PutBucketAclResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_cors(&self, _req: GetBucketCorsRequest) -> Result<GetBucketCorsResponse, Self::Error> { Ok(Default::default()) }
    async fn put_bucket_cors(&self, _req: PutBucketCorsRequest) -> Result<PutBucketCorsResponse, Self::Error> { Ok(Default::default()) }
    async fn delete_bucket_cors(&self, _req: DeleteBucketCorsRequest) -> Result<DeleteBucketCorsResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_website(&self, _req: GetBucketWebsiteRequest) -> Result<GetBucketWebsiteResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_accelerate(&self, _req: GetBucketAccelerateRequest) -> Result<GetBucketAccelerateResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_request_payment(&self, _req: GetBucketRequestPaymentRequest) -> Result<GetBucketRequestPaymentResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_logging(&self, _req: GetBucketLoggingRequest) -> Result<GetBucketLoggingResponse, Self::Error> { Ok(Default::default()) }
    async fn get_bucket_tagging(&self, req: GetBucketTaggingRequest) -> Result<GetBucketTaggingResponse, Self::Error> {
        let p = self.engine().get_bucket_tagging(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketTaggingResponse { xml: p.map(|d| d.body), ..Default::default() })
    }
    async fn delete_bucket_website(&self, _req: DeleteBucketWebsiteRequest) -> Result<DeleteBucketWebsiteResponse, Self::Error> { Ok(Default::default()) }
    async fn delete_bucket_tagging(&self, req: DeleteBucketTaggingRequest) -> Result<DeleteBucketTaggingResponse, Self::Error> {
        self.engine().delete_bucket_tagging(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn list_multipart_uploads(&self, req: ListMultipartUploadsRequest) -> Result<ListMultipartUploadsResponse, Self::Error> {
        let uploads = self.engine().list_multipart_uploads(&req.bucket.bucket, ListOptions::default()).await.map_err(Into::into)?;
        Ok(ListMultipartUploadsResponse {
            uploads: uploads.into_iter().map(|u| MultipartUploadInfo {
                key: u.key,
                upload_id: u.upload_id,
                initiated: Some(u.initiated_at.to_rfc3339()),
            }).collect(),
            ..Default::default()
        })
    }
    async fn list_objects_v2m(&self, req: ListObjectsV2MRequest) -> Result<ListObjectsV2MResponse, Self::Error> {
        let p = self.engine().list_objects_v2(&req.bucket.bucket, ListOptions::default()).await.map_err(Into::into)?;
        Ok(ListObjectsV2MResponse {
            objects: p.objects.iter().map(to_resp_object).collect(),
            ..Default::default()
        })
    }
    async fn list_objects_v2(&self, req: ListObjectsV2Request) -> Result<ListObjectsV2Response, Self::Error> {
        let p = self.engine().list_objects_v2(&req.bucket.bucket, ListOptions::default()).await.map_err(Into::into)?;
        Ok(ListObjectsV2Response { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
    async fn list_object_versions_m(&self, req: ListObjectVersionsMRequest) -> Result<ListObjectVersionsMResponse, Self::Error> {
        let p = self.engine().list_object_versions(&req.bucket.bucket, ListOptions::default()).await.map_err(Into::into)?;
        Ok(ListObjectVersionsMResponse { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
    async fn list_object_versions(&self, req: ListObjectVersionsRequest) -> Result<ListObjectVersionsResponse, Self::Error> {
        let p = self.engine().list_object_versions(&req.bucket.bucket, ListOptions::default()).await.map_err(Into::into)?;
        Ok(ListObjectVersionsResponse { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
    async fn get_bucket_policy_status(&self, req: GetBucketPolicyStatusRequest) -> Result<GetBucketPolicyStatusResponse, Self::Error> {
        let p = self.engine().get_bucket_policy_status(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketPolicyStatusResponse { is_public: Some(p.is_public), ..Default::default() })
    }
    async fn put_bucket_lifecycle(&self, req: PutBucketLifecycleRequest) -> Result<PutBucketLifecycleResponse, Self::Error> {
        self.engine().put_bucket_lifecycle(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_replication_config(&self, req: PutBucketReplicationConfigRequest) -> Result<PutBucketReplicationConfigResponse, Self::Error> {
        self.engine().put_bucket_replication(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_encryption(&self, req: PutBucketEncryptionRequest) -> Result<PutBucketEncryptionResponse, Self::Error> {
        self.engine().put_bucket_encryption(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_policy(&self, req: PutBucketPolicyRequest) -> Result<PutBucketPolicyResponse, Self::Error> {
        self.engine().put_bucket_policy(&req.bucket.bucket, req.json).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_object_lock_config(&self, req: PutBucketObjectLockConfigRequest) -> Result<PutBucketObjectLockConfigResponse, Self::Error> {
        self.engine().put_bucket_object_lock_config(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_tagging(&self, req: PutBucketTaggingRequest) -> Result<PutBucketTaggingResponse, Self::Error> {
        self.engine().put_bucket_tagging(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_versioning(&self, req: PutBucketVersioningRequest) -> Result<PutBucketVersioningResponse, Self::Error> {
        self.engine().put_bucket_versioning(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn put_bucket_notification(&self, req: PutBucketNotificationRequest) -> Result<PutBucketNotificationResponse, Self::Error> {
        self.engine().put_bucket_notification(&req.bucket.bucket, req.xml).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn reset_bucket_replication_start(&self, _req: ResetBucketReplicationStartRequest) -> Result<ResetBucketReplicationStartResponse, Self::Error> { unsupported("ResetBucketReplicationStart") }
    async fn put_bucket(&self, req: PutBucketRequest) -> Result<PutBucketResponse, Self::Error> {
        let _ = self
            .engine()
            .make_bucket(&req.bucket.bucket, req.region.as_deref(), BucketFeatures::default())
            .await
            .map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn head_bucket(&self, req: HeadBucketRequest) -> Result<HeadBucketResponse, Self::Error> {
        let _ = self.engine().head_bucket(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn post_policy(&self, _req: PostPolicyRequest) -> Result<PostPolicyResponse, Self::Error> { unsupported("PostPolicy") }
    async fn delete_multiple_objects(&self, req: DeleteMultipleObjectsRequest) -> Result<DeleteMultipleObjectsResponse, Self::Error> {
        let keys = parse_delete_keys(&req.payload.xml);
        if keys.is_empty() {
            return Err(S3HandlerBridgeError::InvalidRequest("DeleteMultipleObjects payload has no <Key>".to_string()).into());
        }
        let r = self
            .engine()
            .delete_objects(&req.bucket.bucket, keys, DeleteObjectOptions::default())
            .await
            .map_err(Into::into)?;
        Ok(DeleteMultipleObjectsResponse {
            deleted: r.deleted.into_iter().filter_map(|d| d.version_id).collect(),
            errors: r
                .errors
                .into_iter()
                .map(|e| ErrorBody {
                    code: e.code,
                    message: e.message,
                    resource: e.key,
                })
                .collect(),
            ..Default::default()
        })
    }
    async fn delete_bucket_policy(&self, req: DeleteBucketPolicyRequest) -> Result<DeleteBucketPolicyResponse, Self::Error> {
        self.engine().delete_bucket_policy(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn delete_bucket_replication(&self, req: DeleteBucketReplicationRequest) -> Result<DeleteBucketReplicationResponse, Self::Error> {
        self.engine().delete_bucket_replication(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn delete_bucket_lifecycle(&self, req: DeleteBucketLifecycleRequest) -> Result<DeleteBucketLifecycleResponse, Self::Error> {
        self.engine().delete_bucket_lifecycle(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn delete_bucket_encryption(&self, req: DeleteBucketEncryptionRequest) -> Result<DeleteBucketEncryptionResponse, Self::Error> {
        self.engine().delete_bucket_encryption(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn delete_bucket(&self, req: DeleteBucketRequest) -> Result<DeleteBucketResponse, Self::Error> {
        self.engine().delete_bucket(&req.bucket.bucket, false).await.map_err(Into::into)?;
        Ok(Default::default())
    }
    async fn get_bucket_replication_metrics_v2(&self, req: GetBucketReplicationMetricsV2Request) -> Result<GetBucketReplicationMetricsV2Response, Self::Error> {
        let r = self.engine().get_bucket_replication_metrics(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketReplicationMetricsV2Response { json: Some(r.raw_json), ..Default::default() })
    }
    async fn get_bucket_replication_metrics(&self, req: GetBucketReplicationMetricsRequest) -> Result<GetBucketReplicationMetricsResponse, Self::Error> {
        let r = self.engine().get_bucket_replication_metrics(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(GetBucketReplicationMetricsResponse { json: Some(r.raw_json), ..Default::default() })
    }
    async fn validate_bucket_replication_creds(&self, req: ValidateBucketReplicationCredsRequest) -> Result<ValidateBucketReplicationCredsResponse, Self::Error> {
        let v = self.engine().validate_bucket_replication_creds(&req.bucket.bucket).await.map_err(Into::into)?;
        Ok(ValidateBucketReplicationCredsResponse { valid: v.valid, ..Default::default() })
    }
    async fn list_objects_v1(&self, req: ListObjectsV1Request) -> Result<ListObjectsV1Response, Self::Error> {
        let p = self.engine().list_objects_v1(&req.bucket.bucket, ListOptions::default()).await.map_err(Into::into)?;
        Ok(ListObjectsV1Response { objects: p.objects.iter().map(to_resp_object).collect(), ..Default::default() })
    }
}

#[async_trait]
pub trait RootS3Handler
where
    <Self::Engine as S3BucketEngine>::Error: Into<Self::Error>,
    Self::Error: From<S3HandlerBridgeError>,
{
    type Engine: S3BucketEngine;
    type Error: Send + Sync + 'static;
    fn engine(&self) -> &Self::Engine;

    async fn root_listen_notification(
        &self,
        _req: RootListenNotificationRequest,
    ) -> Result<RootListenNotificationResponse, Self::Error> {
        unsupported("RootListenNotification")
    }

    async fn list_buckets(&self, _req: ListBucketsRequest) -> Result<ListBucketsResponse, Self::Error> {
        let list = self.engine().list_buckets().await.map_err(Into::into)?;
        Ok(ListBucketsResponse {
            buckets: list
                .into_iter()
                .map(|b| BucketInfo {
                    name: b.identity.name,
                    creation_date: Some(b.identity.created_at.to_rfc3339()),
                })
                .collect(),
            ..Default::default()
        })
    }

    async fn list_buckets_double_slash(
        &self,
        _req: ListBucketsDoubleSlashRequest,
    ) -> Result<ListBucketsDoubleSlashResponse, Self::Error> {
        let list = self.engine().list_buckets().await.map_err(Into::into)?;
        Ok(ListBucketsDoubleSlashResponse {
            buckets: list
                .into_iter()
                .map(|b| BucketInfo {
                    name: b.identity.name,
                    creation_date: Some(b.identity.created_at.to_rfc3339()),
                })
                .collect(),
            ..Default::default()
        })
    }
}

#[async_trait]
pub trait RejectedS3Handler {
    type Error: Send + Sync + 'static;

    async fn rejected_object_torrent(
        &self,
        req: RejectedObjectTorrentRequest,
    ) -> Result<RejectedApiResponse, Self::Error> {
        Ok(RejectedApiResponse {
            error: ErrorBody {
                code: "NotImplemented".to_string(),
                message: "Object torrent API is not implemented".to_string(),
                resource: Some(format!("{}/{} {}", req.object.bucket, req.object.object, req.method)),
            },
            ..Default::default()
        })
    }
    async fn rejected_object_acl_delete(
        &self,
        req: RejectedObjectAclDeleteRequest,
    ) -> Result<RejectedApiResponse, Self::Error> {
        Ok(RejectedApiResponse {
            error: ErrorBody {
                code: "NotImplemented".to_string(),
                message: "Object ACL delete API is not implemented".to_string(),
                resource: Some(format!("{}/{}", req.object.bucket, req.object.object)),
            },
            ..Default::default()
        })
    }
    async fn rejected_bucket_api(
        &self,
        req: RejectedBucketApiRequest,
    ) -> Result<RejectedApiResponse, Self::Error> {
        Ok(RejectedApiResponse {
            error: ErrorBody {
                code: "NotImplemented".to_string(),
                message: format!("Bucket API {} is not implemented", req.api),
                resource: Some(format!("{} {}", req.bucket.bucket, req.method)),
            },
            ..Default::default()
        })
    }
}
