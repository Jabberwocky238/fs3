use async_trait::async_trait;

use crate::types::{errors::S3EngineError, traits::s3_handler::RootS3Handler};


pub struct S3AxumHandler<Engine> {
    pub engine: Engine,
}

impl<Engine> S3AxumHandler<Engine> {
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }
}

// Handler impls — engine error E is used directly as handler error
impl<Engine, E> RootS3Handler<E> for S3AxumHandler<Engine, E>
where
    Engine: S3BucketEngine<E> + Send + Sync,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    fn engine(&self) -> &Engine { &self.engine }
}

impl<Engine, E> BucketS3Handler<E> for S3AxumHandler<Engine, E>
where
    Engine: S3Engine<E> + Send + Sync,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    fn engine(&self) -> &Engine { &self.engine }
}

impl<Engine, E> ObjectS3Handler<E> for S3AxumHandler<Engine, E>
where
    Engine: S3ObjectEngine<E> + S3MultipartEngine<E> + Send + Sync,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    fn engine(&self) -> &Engine { &self.engine }
}

#[async_trait]
impl<Engine, E> RejectedS3Handler<E> for S3AxumHandler<Engine, E>
where
    Engine: Send + Sync + 'static,
    E: S3EngineError,
{
}