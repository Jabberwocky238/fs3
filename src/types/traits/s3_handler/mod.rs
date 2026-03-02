use async_trait::async_trait;
use chrono::SecondsFormat;

mod bucket;
mod object;
mod utils;

use crate::types::errors::S3EngineError;
use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::S3BucketEngine;
pub use utils::S3HandlerBridgeError;

pub use bucket::BucketS3Handler;
pub use object::ObjectS3Handler;

pub trait S3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>:
    ObjectS3Handler<E>
    + BucketS3Handler<E>
    + RootS3Handler<E>
    + RejectedS3Handler<E>
{
}
impl<T, E: From<S3HandlerBridgeError> + From<S3EngineError>> S3Handler<E> for T
where
    T: ObjectS3Handler<E>
        + BucketS3Handler<E>
        + RootS3Handler<E>
        + RejectedS3Handler<E>,
{
}

// --- Trait definitions ---

#[async_trait]
pub trait RootS3Handler<E: From<S3HandlerBridgeError> + From<S3EngineError>>: Send + Sync {
    type Engine: S3BucketEngine;
    fn engine(&self) -> &Self::Engine;

    async fn root_listen_notification(&self, _req: RootListenNotificationRequest) -> Result<RootListenNotificationResponse, E> {
        utils::unsupported("RootListenNotification")
    }

    async fn list_buckets(&self, _req: ListBucketsRequest) -> Result<ListBucketsResponse, E> {
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