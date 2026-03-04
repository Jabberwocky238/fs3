use async_trait::async_trait;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketTaggingEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait BucketTaggingS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    fn engine(&self) -> &(impl S3BucketTaggingEngine + Send + Sync);
    fn policy(&self) -> &impl S3PolicyEngine;

    async fn get_bucket_tagging(&self, req: GetBucketTaggingRequest) -> Result<GetBucketTaggingResponse, E> {
        check_access(self.policy(), S3Action::GetBucketTagging, Some(&req.bucket.bucket), None).await?;
        let tags = self.engine().get_bucket_tagging(&req.bucket.bucket).await?.unwrap_or_default();
        Ok(GetBucketTaggingResponse { tags, ..Default::default() })
    }

    async fn put_bucket_tagging(&self, req: PutBucketTaggingRequest) -> Result<PutBucketTaggingResponse, E> {
        check_access(self.policy(), S3Action::PutBucketTagging, Some(&req.bucket.bucket), None).await?;
        let tags = parse_tags_xml(&req.xml);
        self.engine().put_bucket_tagging(&req.bucket.bucket, tags).await?;
        Ok(Default::default())
    }

    async fn delete_bucket_tagging(&self, req: DeleteBucketTaggingRequest) -> Result<DeleteBucketTaggingResponse, E> {
        check_access(self.policy(), S3Action::PutBucketTagging, Some(&req.bucket.bucket), None).await?;
        self.engine().delete_bucket_tagging(&req.bucket.bucket).await?;
        Ok(Default::default())
    }
}

fn parse_tags_xml(xml: &str) -> std::collections::HashMap<String, String> {
    let mut tags = std::collections::HashMap::new();
    for tag in xml.split("<Tag>").skip(1) {
        if let Some(end) = tag.find("</Tag>") {
            let content = &tag[..end];
            if let (Some(k), Some(v)) = (
                content.find("<Key>").and_then(|s| content[s+5..].find("</Key>").map(|e| &content[s+5..s+5+e])),
                content.find("<Value>").and_then(|s| content[s+7..].find("</Value>").map(|e| &content[s+7..s+7+e]))
            ) {
                tags.insert(k.to_string(), v.to_string());
            }
        }
    }
    tags
}
