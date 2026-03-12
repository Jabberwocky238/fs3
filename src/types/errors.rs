use std::fmt::{Display, Formatter};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FS3Error {
    status: StatusCode,
    message: String,
}

pub type S3Error = FS3Error;
pub type S3EngineError = FS3Error;
pub type StorageError = FS3Error;

impl Display for FS3Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FS3Error {}

impl FS3Error {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message)
    }

    pub fn precondition_failed(message: impl Into<String>) -> Self {
        Self::new(StatusCode::PRECONDITION_FAILED, message)
    }

    pub fn not_modified(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_MODIFIED, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn method_not_allowed(message: impl Into<String>) -> Self {
        Self::new(StatusCode::METHOD_NOT_ALLOWED, message)
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<&str> for FS3Error {
    fn from(value: &str) -> Self {
        Self::bad_request(value)
    }
}

impl From<String> for FS3Error {
    fn from(value: String) -> Self {
        Self::bad_request(value)
    }
}

impl From<std::io::Error> for FS3Error {
    fn from(value: std::io::Error) -> Self {
        Self::internal(value.to_string())
    }
}

impl From<serde_json::Error> for FS3Error {
    fn from(value: serde_json::Error) -> Self {
        Self::bad_request(value.to_string())
    }
}

impl From<quick_xml::DeError> for FS3Error {
    fn from(value: quick_xml::DeError) -> Self {
        Self::bad_request(value.to_string())
    }
}

impl From<chrono::ParseError> for FS3Error {
    fn from(value: chrono::ParseError) -> Self {
        Self::bad_request(value.to_string())
    }
}

impl IntoResponse for FS3Error {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}
