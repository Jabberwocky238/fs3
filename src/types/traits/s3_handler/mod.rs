use async_trait::async_trait;
use thiserror::Error;

mod bucket;
mod object;
mod utils;

use crate::types::s3::request::*;
use crate::types::s3::response::*;
use crate::types::traits::s3_engine::{S3Engine, S3EngineError};

pub use bucket::BucketS3Handler;
pub use object::ObjectS3Handler;

pub trait S3Handler<E: S3EngineError>:
    ObjectS3Handler<E>
    + BucketS3Handler<E>
    + RootS3Handler<E>
    + RejectedS3Handler<E>
{
}
impl<T, E: S3EngineError> S3Handler<E> for T
where
    T: ObjectS3Handler<E>
        + BucketS3Handler<E>
        + RootS3Handler<E>
        + RejectedS3Handler<E>,
{
}

#[derive(Debug, Error)]
pub enum S3HandlerBridgeError {
    #[error("unsupported by current S3 engine: {0}")]
    Unsupported(&'static str),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
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
    Engine: S3BucketEngine<E>,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    fn engine(&self) -> &Engine { &self.engine }
}

impl<Engine, E> BucketS3Handler<E> for Handler<Engine, E>
where
    Engine: S3Engine<E>,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    fn engine(&self) -> &Engine { &self.engine }
}

impl<Engine, E> ObjectS3Handler<E> for Handler<Engine, E>
where
    Engine: S3ObjectEngine<E> + S3MultipartEngine<E>,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
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

use crate::types::traits::s3_engine::{S3BucketEngine, S3BucketConfigEngine, S3MultipartEngine, S3ObjectEngine};

#[async_trait]
pub trait RootS3Handler<E: S3EngineError + From<S3HandlerBridgeError>> {
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
                creation_date: Some(b.identity.created_at.to_rfc3339()),
            }).collect(),
            ..Default::default()
        })
    }

    async fn list_buckets_double_slash(&self, _req: ListBucketsDoubleSlashRequest) -> Result<ListBucketsDoubleSlashResponse, E> {
        let list = self.engine().list_buckets().await?;
        Ok(ListBucketsDoubleSlashResponse {
            buckets: list.into_iter().map(|b| BucketInfo {
                name: b.identity.name,
                creation_date: Some(b.identity.created_at.to_rfc3339()),
            }).collect(),
            ..Default::default()
        })
    }
}
