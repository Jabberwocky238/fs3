use async_trait::async_trait;
use chrono::SecondsFormat;

mod bucket;
mod object;
mod utils;
mod bucket_lifecycle;
mod bucket_encryption;
mod bucket_object_lock;
mod bucket_versioning;
mod bucket_notification;
mod bucket_replication;
mod bucket_tagging;
mod bucket_website;
mod bucket_cors;
mod object_tagging;
mod object_retention;
mod object_legal_hold;

use crate::types::errors::S3EngineError;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketEngine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::s3::policy::S3Action;
pub use utils::S3HandlerBridgeError;

pub use bucket::BucketS3Handler;
pub use object::ObjectS3Handler;
pub use bucket_lifecycle::BucketLifecycleS3Handler;
pub use bucket_encryption::BucketEncryptionS3Handler;
pub use bucket_object_lock::BucketObjectLockS3Handler;
pub use bucket_versioning::BucketVersioningS3Handler;
pub use bucket_notification::BucketNotificationS3Handler;
pub use bucket_replication::BucketReplicationS3Handler;
pub use bucket_tagging::BucketTaggingS3Handler;
pub use object_tagging::ObjectTaggingS3Handler;
pub use object_retention::ObjectRetentionS3Handler;
pub use object_legal_hold::ObjectLegalHoldS3Handler;
pub use bucket_website::BucketWebsiteS3Handler;
pub use bucket_cors::BucketCorsS3Handler;

pub trait S3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>:
    ObjectS3Handler<E>
    + BucketS3Handler<E>
    + BucketLifecycleS3Handler<E>
    + BucketEncryptionS3Handler<E>
    + BucketObjectLockS3Handler<E>
    + BucketVersioningS3Handler<E>
    + BucketNotificationS3Handler<E>
    + BucketReplicationS3Handler<E>
    + BucketTaggingS3Handler<E>
    + BucketWebsiteS3Handler<E>
    + BucketCorsS3Handler<E>
    + ObjectTaggingS3Handler<E>
    + ObjectRetentionS3Handler<E>
    + ObjectLegalHoldS3Handler<E>
    + RootS3Handler<E>
    + RejectedS3Handler<E>
{
}
impl<T, E: From<S3HandlerBridgeError> + From<S3EngineError>> S3Handler<E> for T
where
    T: ObjectS3Handler<E>
        + BucketS3Handler<E>
        + BucketLifecycleS3Handler<E>
        + BucketEncryptionS3Handler<E>
        + BucketObjectLockS3Handler<E>
        + BucketVersioningS3Handler<E>
        + BucketNotificationS3Handler<E>
        + BucketReplicationS3Handler<E>
        + BucketTaggingS3Handler<E>
        + BucketWebsiteS3Handler<E>
        + BucketCorsS3Handler<E>
        + ObjectTaggingS3Handler<E>
        + ObjectRetentionS3Handler<E>
        + ObjectLegalHoldS3Handler<E>
        + RootS3Handler<E>
        + RejectedS3Handler<E>,
{
}

// --- Trait definitions ---

#[async_trait]
pub trait RootS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    type Engine: S3BucketEngine;
    type Policy: S3PolicyEngine;
    fn engine(&self) -> &Self::Engine;
    fn policy(&self) -> &Self::Policy;

    async fn root_listen_notification(&self, _req: RootListenNotificationRequest) -> Result<RootListenNotificationResponse, E> {
        utils::unsupported("RootListenNotification")
    }

    async fn list_buckets(&self, _req: ListBucketsRequest) -> Result<ListBucketsResponse, E> {
        utils::check_access(self.policy(), S3Action::ListAllMyBuckets, None, None).await?;
        let list = self.engine().list_buckets().await?;
        Ok(ListBucketsResponse {
            buckets: list.into_iter().map(|b| BucketInfo {
                name: b.identity.name,
                creation_date: Some(b.identity.created_at.to_rfc3339_opts(SecondsFormat::Secs, true)),
            }).collect(),
            ..Default::default()
        })
    }

    async fn list_buckets_double_slash(&self, _req: ListBucketsDoubleSlashRequest) -> Result<ListBucketsDoubleSlashResponse, E> {
        utils::check_access(self.policy(), S3Action::ListAllMyBuckets, None, None).await?;
        let list = self.engine().list_buckets().await?;
        Ok(ListBucketsDoubleSlashResponse {
            buckets: list.into_iter().map(|b| BucketInfo {
                name: b.identity.name,
                creation_date: Some(b.identity.created_at.to_rfc3339_opts(SecondsFormat::Secs, true)),
            }).collect(),
            ..Default::default()
        })
    }
}

#[async_trait]
pub trait RejectedS3Handler<E>: Send + Sync {
    async fn rejected_object_torrent(&self, req: RejectedObjectTorrentRequest) -> Result<RejectedApiResponse, E> {
        Ok(RejectedApiResponse {
            error: ErrorBody {
                code: "NotImplemented".to_string(),
                message: "Object torrent API is not implemented".to_string(),
                resource: Some(format!("{}/{} {}", req.object.bucket, req.object.object, req.method)),
            },
            ..Default::default()
        })
    }
    async fn rejected_object_acl_delete(&self, req: RejectedObjectAclDeleteRequest) -> Result<RejectedApiResponse, E> {
        Ok(RejectedApiResponse {
            error: ErrorBody {
                code: "NotImplemented".to_string(),
                message: "Object ACL delete API is not implemented".to_string(),
                resource: Some(format!("{}/{}", req.object.bucket, req.object.object)),
            },
            ..Default::default()
        })
    }
    async fn rejected_bucket_api(&self, req: RejectedBucketApiRequest) -> Result<RejectedApiResponse, E> {
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