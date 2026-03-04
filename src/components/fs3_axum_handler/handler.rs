use async_trait::async_trait;

use crate::types::errors::S3EngineError;
use crate::types::traits::s3_engine::S3Engine;
use crate::types::traits::s3_policyengine::S3PolicyEngine;
use crate::types::traits::s3_handler::{
    BucketS3Handler, ObjectS3Handler, RejectedS3Handler, RootS3Handler, S3HandlerBridgeError,
    BucketLifecycleS3Handler, BucketEncryptionS3Handler, BucketObjectLockS3Handler,
    BucketVersioningS3Handler, BucketNotificationS3Handler, BucketReplicationS3Handler,
    BucketTaggingS3Handler, ObjectTaggingS3Handler, ObjectRetentionS3Handler, ObjectLegalHoldS3Handler,
};

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
    fn engine(&self) -> &Engine { &self.engine }
    fn policy(&self) -> &Policy { &self.policy }
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
    fn engine(&self) -> &Engine { &self.engine }
    fn policy(&self) -> &Policy { &self.policy }
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
    fn engine(&self) -> &Engine { &self.engine }
    fn policy(&self) -> &Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> RejectedS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: Send + 'static,
{
}

#[async_trait]
impl<Engine, Policy, E> BucketLifecycleS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn bucket_lifecycle_engine_provider(&self) -> &Engine { &self.engine }
    fn bucket_lifecycle_policy_provider(&self) -> &Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> BucketEncryptionS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn bucket_encryption_engine_provider(&self) -> &Engine { &self.engine }
    fn bucket_encryption_policy_provider(&self) -> &Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> BucketObjectLockS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn bucket_object_lock_engine_provider(&self) -> &Engine { &self.engine }
    fn bucket_object_lock_policy_provider(&self) -> &Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> BucketVersioningS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn bucket_versioning_engine_provider(&self) -> &Engine { &self.engine }
    fn bucket_versioning_policy_provider(&self) -> &Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> BucketNotificationS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn bucket_notification_engine_provider(&self) -> &Engine { &self.engine }
    fn bucket_notification_policy_provider(&self) -> &Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> BucketReplicationS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn bucket_replication_engine_provider(&self) -> &Engine { &self.engine }
    fn bucket_replication_policy_provider(&self) -> &Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> BucketTaggingS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn bucket_tagging_engine_provider(&self) -> &Engine { &self.engine }
    fn bucket_tagging_policy_provider(&self) -> &Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> ObjectTaggingS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn object_tagging_engine_provider(&self) -> &Engine { &self.engine }
    fn object_tagging_policy_provider(&self) -> &Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> ObjectRetentionS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn object_retention_engine_provider(&self) -> &Engine { &self.engine }
    fn object_retention_policy_provider(&self) -> &Policy { &self.policy }
}

#[async_trait]
impl<Engine, Policy, E> ObjectLegalHoldS3Handler<E> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine + Send + Sync,
    Policy: S3PolicyEngine + Send + Sync,
    E: From<S3HandlerBridgeError> + From<S3EngineError> + Send + 'static,
{
    type Engine = Engine;
    type Policy = Policy;
    fn object_legal_hold_engine_provider(&self) -> &Engine { &self.engine }
    fn object_legal_hold_policy_provider(&self) -> &Policy { &self.policy }
}
