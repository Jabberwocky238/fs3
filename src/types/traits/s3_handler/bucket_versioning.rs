use async_trait::async_trait;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketVersionEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketVersioningS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    type Engine: S3BucketVersionEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn bucket_versioning_engine_provider(&self) -> &Self::Engine;
    fn bucket_versioning_policy_provider(&self) -> &Self::Policy;

    async fn get_bucket_versioning(&self, req: GetBucketVersioningRequest) -> Result<GetBucketVersioningResponse, E> {
        check_access(self.bucket_versioning_policy_provider(), S3Action::GetBucketVersioning, Some(&req.bucket.bucket), None).await?;
        let v = self.bucket_versioning_engine_provider().get_bucket_versioning(&req.bucket.bucket).await?;
        Ok(GetBucketVersioningResponse {
            status: v.as_ref().map(|x| x.status.clone()),
            mfa_delete: v.as_ref().and_then(|x| x.mfa_delete.clone()),
            ..Default::default()
        })
    }

    async fn put_bucket_versioning(&self, req: PutBucketVersioningRequest) -> Result<PutBucketVersioningResponse, E> {
        check_access(self.bucket_versioning_policy_provider(), S3Action::PutBucketVersioning, Some(&req.bucket.bucket), None).await?;
        let (status, mfa_delete) = parse_versioning_config(&req.xml)?;
        self.bucket_versioning_engine_provider().put_bucket_versioning(&req.bucket.bucket, status, mfa_delete).await?;
        Ok(Default::default())
    }
}

fn parse_versioning_config(xml: &str) -> Result<(String, Option<String>), S3HandlerBridgeError> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct VersioningConfiguration {
        #[serde(rename = "Status")]
        status: Option<String>,
    }

    let config: VersioningConfiguration = serde_xml_rs::from_str(xml)
        .map_err(|e| S3HandlerBridgeError::InvalidVersioningStatus(format!("XML parse error: {}", e)))?;

    config.status
        .ok_or_else(|| S3HandlerBridgeError::InvalidVersioningStatus("Missing VersioningStatus".to_string()))
        .map(|s| (s, None))
}
