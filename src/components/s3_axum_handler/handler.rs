use async_trait::async_trait;

use crate::types::errors::S3EngineError;
use crate::types::traits::s3_engine::{S3BucketEngine, S3BucketConfigEngine, S3ObjectEngine, S3MultipartEngine};
use crate::types::traits::s3_handler::{
    S3HandlerBridgeError, BucketS3Handler, ObjectS3Handler, RootS3Handler, RejectedS3Handler,
};

pub struct S3AxumHandler<Engine> {
    pub engine: Engine,
}

impl<Engine> S3AxumHandler<Engine> {
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }
}
