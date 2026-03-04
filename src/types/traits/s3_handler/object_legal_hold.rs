use async_trait::async_trait;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3ObjectLegalHoldEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait ObjectLegalHoldS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    type Engine: S3ObjectLegalHoldEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn object_legal_hold_engine_provider(&self) -> &Self::Engine;
    fn object_legal_hold_policy_provider(&self) -> &Self::Policy;

    async fn get_object_legal_hold(&self, req: GetObjectLegalHoldRequest) -> Result<GetObjectLegalHoldResponse, E> {
        check_access(self.object_legal_hold_policy_provider(), S3Action::GetObjectLegalHold, Some(&req.bucket.bucket), Some(&req.object.object)).await?;
        let _h = self.object_legal_hold_engine_provider().get_object_legal_hold(&req.bucket.bucket, &req.object.object).await?;
        Ok(GetObjectLegalHoldResponse { ..Default::default() })
    }

    async fn put_object_legal_hold(&self, req: PutObjectLegalHoldRequest) -> Result<PutObjectLegalHoldResponse, E> {
        check_access(self.object_legal_hold_policy_provider(), S3Action::PutObjectLegalHold, Some(&req.bucket.bucket), Some(&req.object.object)).await?;
        let legal_hold = parse_legal_hold(&req.xml);
        self.object_legal_hold_engine_provider().put_object_legal_hold(&req.bucket.bucket, &req.object.object, legal_hold).await?;
        Ok(Default::default())
    }
}

fn parse_legal_hold(_xml: &str) -> crate::types::s3::core::ObjectLegalHold {
    crate::types::s3::core::ObjectLegalHold { enabled: false }
}
