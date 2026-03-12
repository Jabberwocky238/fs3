use async_trait::async_trait;

use crate::types::FS3Error;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3ObjectLegalHoldEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;

use super::utils::*;

#[async_trait]
pub trait ObjectLegalHoldS3Handler: Send + Sync {
    type Engine: S3ObjectLegalHoldEngine<FS3Error> + Send + Sync;
    type Policy: S3PolicyEngine<FS3Error> + Send + Sync;
    fn object_legal_hold_engine_provider(&self) -> &Self::Engine;
    fn object_legal_hold_policy_provider(&self) -> &Self::Policy;

    async fn get_object_legal_hold(&self, req: GetObjectLegalHoldRequest) -> Result<GetObjectLegalHoldResponse, FS3Error> {
        check_access(self.object_legal_hold_policy_provider(), S3Action::GetObjectLegalHold, Some(&req.bucket.bucket), Some(&req.object.object)).await?;
        let _h = self.object_legal_hold_engine_provider().get_object_legal_hold(&req.bucket.bucket, &req.object.object).await?;
        Ok(GetObjectLegalHoldResponse { ..Default::default() })
    }

    async fn put_object_legal_hold(&self, req: PutObjectLegalHoldRequest) -> Result<PutObjectLegalHoldResponse, FS3Error> {
        check_access(self.object_legal_hold_policy_provider(), S3Action::PutObjectLegalHold, Some(&req.bucket.bucket), Some(&req.object.object)).await?;
        self.object_legal_hold_engine_provider().put_object_legal_hold(&req.bucket.bucket, &req.object.object, req.legal_hold).await?;
        Ok(Default::default())
    }
}

