pub mod object_layer;
pub mod s3_engine;
pub mod s3_handler;
pub mod s3_policyengine;
pub mod storage_access;
pub mod storage_api;

pub trait StdError: std::error::Error + Send + Sync + 'static {}

impl<T> StdError for T where T: std::error::Error + Send + Sync + 'static {}
