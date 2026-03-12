use crate::types::errors::FS3Error;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_handler::utils;
use crate::types::traits::s3_engine::{S3BucketEngine, S3BucketConfigEngine, S3BucketWebsiteEngine};
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use async_trait::async_trait;
use crate::types::traits::BoxError;

#[async_trait]
pub trait BucketCorsS3Handler: Send + Sync {
    type Engine: S3BucketEngine + S3BucketConfigEngine + S3BucketWebsiteEngine;
    type Policy: S3PolicyEngine;
    fn engine(&self) -> &Self::Engine;
    fn policy(&self) -> &Self::Policy;

    async fn get_bucket_cors(&self, req: GetBucketCorsRequest) -> Result<GetBucketCorsResponse , BoxError> {
        utils::check_access(self.policy(), S3Action::GetBucketCors, Some(&req.bucket.bucket), None).await?;
        let meta = self.engine().get_bucket_metadata(&req.bucket.bucket).await?;
        let cors = meta.cors.ok_or_else(|| FS3Error::from("No such CORS configuration"))?;
        Ok(GetBucketCorsResponse {
            meta: Default::default(),
            cors_rules: cors.rules.iter().map(|rule| {
                let origins = rule.allowed_origins.iter().map(|o| format!("<AllowedOrigin>{}</AllowedOrigin>", o)).collect::<String>();
                let methods = rule.allowed_methods.iter().map(|m| format!("<AllowedMethod>{}</AllowedMethod>", m)).collect::<String>();
                let headers = rule.allowed_headers.iter().map(|h| format!("<AllowedHeader>{}</AllowedHeader>", h)).collect::<String>();
                let expose = rule.expose_headers.iter().map(|e| format!("<ExposeHeader>{}</ExposeHeader>", e)).collect::<String>();
                let max_age = rule.max_age_seconds.map(|s| format!("<MaxAgeSeconds>{}</MaxAgeSeconds>", s)).unwrap_or_default();
                format!("<CORSRule>{}{}{}{}{}</CORSRule>", origins, methods, headers, expose, max_age)
            }).collect()
        })
    }

    async fn put_bucket_cors(&self, req: PutBucketCorsRequest) -> Result<PutBucketCorsResponse , BoxError> {
        utils::check_access(self.policy(), S3Action::PutBucketCors, Some(&req.bucket.bucket), None).await?;
        self.engine().set_bucket_cors(&req.bucket.bucket, Some(req.cors)).await?;
        Ok(PutBucketCorsResponse { meta: Default::default() })
    }

    async fn delete_bucket_cors(&self, req: DeleteBucketCorsRequest) -> Result<DeleteBucketCorsResponse , BoxError> {
        utils::check_access(self.policy(), S3Action::DeleteBucketCors, Some(&req.bucket.bucket), None).await?;
        self.engine().set_bucket_cors(&req.bucket.bucket, None).await?;
        Ok(DeleteBucketCorsResponse { meta: Default::default() })
    }
}

