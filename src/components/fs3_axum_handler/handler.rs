use async_trait::async_trait;

use crate::types::errors::S3EngineError;
use crate::types::traits::s3_engine::S3Engine;
use crate::types::traits::s3_handler::{BucketS3Handler, ObjectS3Handler, RejectedS3Handler, RootS3Handler, S3HandlerBridgeError};

pub struct S3AxumHandler<Engine: S3Engine> {
    pub engine: Engine,
}

impl<Engine: S3Engine> S3AxumHandler<Engine> {
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl<Engine, E> ObjectS3Handler<E> for S3AxumHandler<Engine>
where
    Engine: S3Engine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    fn engine(&self) -> &Self::Engine { &self.engine }
}

#[async_trait]
impl<Engine, E> BucketS3Handler<E> for S3AxumHandler<Engine>
where
    Engine: S3Engine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    fn engine(&self) -> &Self::Engine { &self.engine }
}

#[async_trait]
impl<Engine, E> RootS3Handler<E> for S3AxumHandler<Engine>
where
    Engine: S3Engine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    fn engine(&self) -> &Self::Engine { &self.engine }
}

#[async_trait]
impl<Engine, E> RejectedS3Handler<E> for S3AxumHandler<Engine>
where
    Engine: S3Engine + Send + Sync,
    E: Send + 'static,
{
}
