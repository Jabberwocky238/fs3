pub mod request;
pub mod response;

pub use request::*;
pub use response::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct S3Binding {
    pub request: S3Request,
    pub response: S3Response,
}
