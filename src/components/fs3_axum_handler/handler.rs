use async_trait::async_trait;

use crate::types::errors::S3EngineError;
use crate::types::traits::s3_engine::S3Engine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::traits::s3_handler::{BucketS3Handler, ObjectS3Handler, RejectedS3Handler, RootS3Handler, S3HandlerBridgeError};

pub struct S3AxumHandler<Engine: S3Engine, Policy: S3PolicyEngine> {
    pub engine: Engine,
    pub policy: Policy,
}

impl<Engine: S3Engine, Policy: S3PolicyEngine> S3AxumHandler<Engine, Policy> {
    pub fn new(engine: Engine, policy: Policy) -> Self {
        Self { engine, policy }
    }
}

#[async_trait]
impl<Engine, Policy, E> ObjectS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn engine(&self) -> &Self::Engine { &self.engine }
    fn policy(&self) -> &Self::Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> BucketS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn engine(&self) -> &Self::Engine { &self.engine }
    fn policy(&self) -> &Self::Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> RootS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn engine(&self) -> &Self::Engine { &self.engine }
    fn policy(&self) -> &Self::Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> RejectedS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: Send + 'static,
{
}
