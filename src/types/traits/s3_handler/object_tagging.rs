use async_trait::async_trait;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3ObjectTaggingEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
use crate::types::errors::S3EngineError;
use super::utils::*;

#[async_trait]
pub trait ObjectTaggingS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    type Engine: S3ObjectTaggingEngine + Send + Sync;
    type Policy: S3PolicyEngine + Send + Sync;
    fn object_tagging_engine_provider(&self) -> &Self::Engine;
    fn object_tagging_policy_provider(&self) -> &Self::Policy;

    async fn get_object_tagging(&self, req: GetObjectTaggingRequest) -> Result<GetObjectTaggingResponse, E> {
        check_access(self.object_tagging_policy_provider(), S3Action::GetObjectTagging, Some(&req.bucket.bucket), Some(&req.object.key)).await?;
        let tags = self.object_tagging_engine_provider().get_object_tagging(&req.bucket.bucket, &req.object.key).await?;
        Ok(GetObjectTaggingResponse { tags, ..Default::default() })
    }

    async fn put_object_tagging(&self, req: PutObjectTaggingRequest) -> Result<PutObjectTaggingResponse, E> {
        check_access(self.object_tagging_policy_provider(), S3Action::PutObjectTagging, Some(&req.bucket.bucket), Some(&req.object.key)).await?;
        let tags = parse_tags_xml(&req.xml);
        self.object_tagging_engine_provider().put_object_tagging(&req.bucket.bucket, &req.object.key, tags).await?;
        Ok(Default::default())
    }

    async fn delete_object_tagging(&self, req: DeleteObjectTaggingRequest) -> Result<DeleteObjectTaggingResponse, E> {
        check_access(self.object_tagging_policy_provider(), S3Action::DeleteObjectTagging, Some(&req.bucket.bucket), Some(&req.object.key)).await?;
        self.object_tagging_engine_provider().delete_object_tagging(&req.bucket.bucket, &req.object.key).await?;
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
