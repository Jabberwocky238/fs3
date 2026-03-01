pub mod core;
pub mod request;
pub mod response;

// JW238: never export *, items should be imported by crate itself, only use * is allowed.
// pub use core::*;
// pub use request::*;
// pub use response::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct S3Binding {
    pub request: request::S3Request,
    pub response: response::S3Response,
}
