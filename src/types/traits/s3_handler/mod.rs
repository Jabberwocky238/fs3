use async_trait::async_trait;
use thiserror::Error;

mod bucket;
mod object;
mod utils;

use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{S3BucketEngine, S3ObjectEngine, S3MultipartEngine, S3BucketConfigEngine, S3Engine};

pub trait S3Handler:
    ObjectS3Handler
    + BucketS3Handler
    + RootS3Handler
    + RejectedObjectS3Handler
    + RejectedBucketS3Handler
{
}
impl<T> S3Handler for T where
    T: ObjectS3Handler
        + BucketS3Handler
        + RootS3Handler
        + RejectedObjectS3Handler
        + RejectedBucketS3Handler
{
}

pub use bucket::BucketS3Handler;
pub use object::ObjectS3Handler;

#[derive(Debug, Error)]
pub enum S3HandlerBridgeError {
    #[error("unsupported by current S3 engine: {0}")]
    Unsupported(&'static str),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

pub struct Handler<Engine, Error>
where
    Engine: S3Engine,
    Error: Send + Sync + 'static,
{
    pub engine: Engine,
    pub _error: std::marker::PhantomData<Error>,
}

impl<Engine, Error> Handler<Engine, Error>
where
    Engine: S3Engine,
    Error: Send + Sync + 'static,
{
    pub fn new(engine: Engine) -> Self {
        Self {
            engine,
            _error: std::marker::PhantomData,
        }
    }
}

impl<Engine, Error> RootS3Handler for Handler<Engine, Error>
where
    Engine: S3Engine,
    Error: Send + Sync + 'static,
    <Engine as S3Engine>::Error: Into<Error>,
    Error: From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    type Error = Error;

    fn engine(&self) -> &Self::Engine {
        &self.engine
    }
}

impl<Engine, Error> BucketS3Handler for Handler<Engine, Error>
where
    Engine: S3BucketEngine,
    Error: Send + Sync + 'static,
    <Engine as S3BucketEngine>::Error: Into<Error>,
    Error: From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    type Error = Error;

    fn engine(&self) -> &Self::Engine {
        &self.engine
    }
}

impl<Engine, Error> ObjectS3Handler for Handler<Engine, Error>
where
    Engine: S3BucketEngine,
    Error: Send + Sync + 'static,
    <Engine as S3BucketEngine>::Error: Into<Error>,
    Error: From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    type Error = Error;

    fn engine(&self) -> &Self::Engine {
        &self.engine
    }
}

#[async_trait]
pub trait RootS3Handler
where
    <Self::Engine as S3BucketEngine>::Error: Into<Self::Error>,
    Self::Error: From<S3HandlerBridgeError>,
{
    type Engine: S3BucketEngine;
    type Error: Send + Sync + 'static;
    fn engine(&self) -> &Self::Engine;

    async fn root_listen_notification(
        &self,
        _req: RootListenNotificationRequest,
    ) -> Result<RootListenNotificationResponse, Self::Error> {
        utils::unsupported::<RootListenNotificationResponse, Self::Error>("RootListenNotification")
    }

    async fn list_buckets(
        &self,
        _req: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, Self::Error> {
        let list = self.engine().list_buckets().await.map_err(Into::into)?;
        Ok(ListBucketsResponse {
            buckets: list
                .into_iter()
                .map(|b| BucketInfo {
                    name: b.identity.name,
                    creation_date: Some(b.identity.created_at.to_rfc3339()),
                })
                .collect(),
            ..Default::default()
        })
    }

    async fn list_buckets_double_slash(
        &self,
        _req: ListBucketsDoubleSlashRequest,
    ) -> Result<ListBucketsDoubleSlashResponse, Self::Error> {
        let list = self.engine().list_buckets().await.map_err(Into::into)?;
        Ok(ListBucketsDoubleSlashResponse {
            buckets: list
                .into_iter()
                .map(|b| BucketInfo {
                    name: b.identity.name,
                    creation_date: Some(b.identity.created_at.to_rfc3339()),
                })
                .collect(),
            ..Default::default()
        })
    }
}

#[async_trait]
pub trait RejectedObjectS3Handler {
    type Error: Send + Sync + 'static;

    async fn rejected_object_torrent(
        &self,
        req: RejectedObjectTorrentRequest,
    ) -> Result<RejectedApiResponse, Self::Error> {
        Ok(RejectedApiResponse {
            error: ErrorBody {
                code: "NotImplemented".to_string(),
                message: "Object torrent API is not implemented".to_string(),
                resource: Some(format!(
                    "{}/{} {}",
                    req.object.bucket, req.object.object, req.method
                )),
            },
            ..Default::default()
        })
    }
    async fn rejected_object_acl_delete(
        &self,
        req: RejectedObjectAclDeleteRequest,
    ) -> Result<RejectedApiResponse, Self::Error> {
        Ok(RejectedApiResponse {
            error: ErrorBody {
                code: "NotImplemented".to_string(),
                message: "Object ACL delete API is not implemented".to_string(),
                resource: Some(format!("{}/{}", req.object.bucket, req.object.object)),
            },
            ..Default::default()
        })
    }
}

#[async_trait]
pub trait RejectedBucketS3Handler {
    type Error: Send + Sync + 'static;

    async fn rejected_bucket_api(
        &self,
        req: RejectedBucketApiRequest,
    ) -> Result<RejectedApiResponse, Self::Error> {
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
