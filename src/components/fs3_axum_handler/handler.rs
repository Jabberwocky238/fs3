use async_trait::async_trait;

use crate::types::FS3Error;
use crate::types::traits::s3_engine::S3Engine;
use crate::types::traits::s3_handler::{
    BucketS3Handler,
    ObjectS3Handler,
    RejectedS3Handler,
    RootS3Handler,
    // BucketLifecycleS3Handler, BucketEncryptionS3Handler, BucketObjectLockS3Handler,
    // BucketVersioningS3Handler, BucketNotificationS3Handler, BucketReplicationS3Handler,
    // BucketTaggingS3Handler, BucketWebsiteS3Handler, BucketCorsS3Handler, ObjectTaggingS3Handler, ObjectRetentionS3Handler, ObjectLegalHoldS3Handler,
};
use crate::types::traits::s3_policyengine::S3PolicyEngine;

pub struct S3AxumHandler<Engine: S3Engine<FS3Error>, Policy: S3PolicyEngine<FS3Error>> {
    pub engine: Engine,
    pub policy: Policy,
}

impl<Engine: S3Engine<FS3Error>, Policy: S3PolicyEngine<FS3Error>> S3AxumHandler<Engine, Policy> {
    pub fn new(engine: Engine, policy: Policy) -> Self {
        Self { engine, policy }
    }
}

#[async_trait]
impl<Engine, Policy> ObjectS3Handler<FS3Error> for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine<FS3Error> + Send + Sync,
    Policy: S3PolicyEngine<FS3Error> + Send + Sync,
{
    type Engine = Engine;
    type Policy = Policy;
    fn engine(&self) -> &Engine {
        &self.engine
    }
    fn policy(&self) -> &Policy {
        &self.policy
    }
}

#[async_trait]
impl<Engine, Policy> BucketS3Handler for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine<FS3Error> + Send + Sync,
    Policy: S3PolicyEngine<FS3Error> + Send + Sync,
{
    type Engine = Engine;
    type Policy = Policy;
    fn engine(&self) -> &Engine {
        &self.engine
    }
    fn policy(&self) -> &Policy {
        &self.policy
    }
}

#[async_trait]
impl<Engine, Policy> RootS3Handler for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine<FS3Error> + Send + Sync,
    Policy: S3PolicyEngine<FS3Error> + Send + Sync,
{
    type Engine = Engine;
    type Policy = Policy;
    fn engine(&self) -> &Engine {
        &self.engine
    }
    fn policy(&self) -> &Policy {
        &self.policy
    }
}

#[async_trait]
impl<Engine, Policy> RejectedS3Handler for S3AxumHandler<Engine, Policy>
where
    Engine: S3Engine<FS3Error> + Send + Sync,
    Policy: S3PolicyEngine<FS3Error> + Send + Sync,
{
}

// #[async_trait]
// impl<Engine, Policy> BucketLifecycleS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn bucket_lifecycle_engine_provider(&self) -> &Engine { &self.engine }
//     fn bucket_lifecycle_policy_provider(&self) -> &Policy { &self.policy }
// }

// #[async_trait]
// impl<Engine, Policy> BucketEncryptionS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn bucket_encryption_engine_provider(&self) -> &Engine { &self.engine }
//     fn bucket_encryption_policy_provider(&self) -> &Policy { &self.policy }
// }

// #[async_trait]
// impl<Engine, Policy> BucketObjectLockS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn bucket_object_lock_engine_provider(&self) -> &Engine { &self.engine }
//     fn bucket_object_lock_policy_provider(&self) -> &Policy { &self.policy }
// }

// #[async_trait]
// impl<Engine, Policy> BucketVersioningS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn bucket_versioning_engine_provider(&self) -> &Engine { &self.engine }
//     fn bucket_versioning_policy_provider(&self) -> &Policy { &self.policy }
// }

// #[async_trait]
// impl<Engine, Policy> BucketNotificationS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn bucket_notification_engine_provider(&self) -> &Engine { &self.engine }
//     fn bucket_notification_policy_provider(&self) -> &Policy { &self.policy }
// }

// #[async_trait]
// impl<Engine, Policy> BucketReplicationS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn bucket_replication_engine_provider(&self) -> &Engine { &self.engine }
//     fn bucket_replication_policy_provider(&self) -> &Policy { &self.policy }
// }

// #[async_trait]
// impl<Engine, Policy> BucketTaggingS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn bucket_tagging_engine_provider(&self) -> &Engine { &self.engine }
//     fn bucket_tagging_policy_provider(&self) -> &Policy { &self.policy }
// }

// #[async_trait]
// impl<Engine, Policy> ObjectTaggingS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn object_tagging_engine_provider(&self) -> &Engine { &self.engine }
//     fn object_tagging_policy_provider(&self) -> &Policy { &self.policy }
// }

// #[async_trait]
// impl<Engine, Policy> ObjectRetentionS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn object_retention_engine_provider(&self) -> &Engine { &self.engine }
//     fn object_retention_policy_provider(&self) -> &Policy { &self.policy }
// }

// #[async_trait]
// impl<Engine, Policy> ObjectLegalHoldS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn object_legal_hold_engine_provider(&self) -> &Engine { &self.engine }
//     fn object_legal_hold_policy_provider(&self) -> &Policy { &self.policy }
// }

// #[async_trait]
// impl<Engine, Policy> BucketWebsiteS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn bucket_website_engine_provider(&self) -> &Engine { &self.engine }
//     fn bucket_website_policy_provider(&self) -> &Policy { &self.policy }
// }

// #[async_trait]
// impl<Engine, Policy> BucketCorsS3Handler for S3AxumHandler<Engine, Policy>
// where
//     Engine: S3Engine + Send + Sync,
//     Policy: S3PolicyEngine + Send + Sync,
// {
//     type Engine = Engine;
//     type Policy = Policy;
//     fn engine(&self) -> &Engine { &self.engine }
//     fn policy(&self) -> &Policy { &self.policy }
// }
