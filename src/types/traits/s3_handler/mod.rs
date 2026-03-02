use async_trait::async_trait;
use chrono::SecondsFormat;

mod bucket;
mod object;
mod utils;

use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{S3BucketEngine, S3Engine, S3EngineError, S3MultipartEngine, S3ObjectEngine};
pub use utils::S3HandlerBridgeError;

pub use bucket::BucketS3Handler;
pub use object::ObjectS3Handler;

pub trait S3Handler<E: S3EngineError + From<S3HandlerBridgeError>>:
    ObjectS3Handler<E>
    + BucketS3Handler<E>
    + RootS3Handler<E>
    + RejectedS3Handler<E>
{
}
impl<T, E: S3EngineError + From<S3HandlerBridgeError>> S3Handler<E> for T
where
    T: ObjectS3Handler<E>
        + BucketS3Handler<E>
        + RootS3Handler<E>
        + RejectedS3Handler<E>,
{
}

pub struct Handler<Engine, E: S3EngineError> {
    pub engine: Engine,
    pub _error: std::marker::PhantomData<E>,
}

impl<Engine, E: S3EngineError> Handler<Engine, E> {
    pub fn new(engine: Engine) -> Self {
        Self { engine, _error: std::marker::PhantomData }
    }
}

// Handler impls — engine error E is used directly as handler error
impl<Engine, E> RootS3Handler<E> for Handler<Engine, E>
where
    Engine: S3BucketEngine<E> + Send + Sync,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    fn engine(&self) -> &Engine { &self.engine }
}

impl<Engine, E> BucketS3Handler<E> for Handler<Engine, E>
where
    Engine: S3Engine<E> + Send + Sync,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    fn engine(&self) -> &Engine { &self.engine }
}

impl<Engine, E> ObjectS3Handler<E> for Handler<Engine, E>
where
    Engine: S3ObjectEngine<E> + S3MultipartEngine<E> + Send + Sync,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    fn engine(&self) -> &Engine { &self.engine }
}

#[async_trait]
impl<Engine, E> RejectedS3Handler<E> for Handler<Engine, E>
where
    Engine: Send + Sync + 'static,
    E: S3EngineError,
{
}

// --- Trait definitions ---

#[async_trait]
pub trait RootS3Handler<E: S3EngineError + From<S3HandlerBridgeError>>: Send + Sync {
    type Engine: S3BucketEngine<E>;
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
pub trait RejectedS3Handler<E: S3EngineError>: Send + Sync {
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