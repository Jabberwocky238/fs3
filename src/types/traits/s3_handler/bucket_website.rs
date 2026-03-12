use async_trait::async_trait;

use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketWebsiteEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;

use super::utils::*;

#[async_trait]
pub trait BucketWebsiteS3Handler: Send + Sync {
    type Engine: S3BucketWebsiteEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn bucket_website_engine_provider(&self) -> &Self::Engine;
    fn bucket_website_policy_provider(&self) -> &Self::Policy;

    async fn get_bucket_website(&self, req: GetBucketWebsiteRequest) -> Result<GetBucketWebsiteResponse , BoxError> {
        check_access(self.bucket_website_policy_provider(), S3Action::GetBucketWebsite, Some(&req.bucket.bucket), None).await?;
        let config = self.bucket_website_engine_provider().get_bucket_website(&req.bucket.bucket).await?;

        let (index_doc, error_doc) = if let Some(xml) = config {
            let index = xml.find("<Suffix>").and_then(|start| {
                let end = xml[start..].find("</Suffix>")?;
                Some(xml[start+8..start+end].to_string())
            });
            let error = xml.find("<Key>").and_then(|start| {
                let end = xml[start..].find("</Key>")?;
                Some(xml[start+5..start+end].to_string())
            });
            (index, error)
        } else {
            (None, None)
        };

        Ok(GetBucketWebsiteResponse {
            meta: Default::default(),
            index_document: index_doc,
            error_document: error_doc,
        })
    }

    async fn put_bucket_website(&self, req: PutBucketWebsiteRequest) -> Result<PutBucketWebsiteResponse , BoxError> {
        check_access(self.bucket_website_policy_provider(), S3Action::PutBucketWebsite, Some(&req.bucket.bucket), None).await?;
        let mut website = format!(
            "<WebsiteConfiguration><IndexDocument><Suffix>{}</Suffix></IndexDocument>",
            req.website.index_document
        );
        if let Some(error_document) = req.website.error_document {
            website.push_str(&format!("<ErrorDocument><Key>{error_document}</Key></ErrorDocument>"));
        }
        website.push_str("</WebsiteConfiguration>");
        self.bucket_website_engine_provider().put_bucket_website(&req.bucket.bucket, website).await?;
        Ok(PutBucketWebsiteResponse { meta: Default::default() })
    }
}

