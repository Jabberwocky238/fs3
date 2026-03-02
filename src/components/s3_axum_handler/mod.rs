mod util;
mod http_bucket;
mod http_object;
mod http_root;

use std::fmt::{Display, Formatter};
use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use axum::Router;

use crate::types::traits::s3_engine::S3EngineError;
use crate::types::traits::s3_handler::{S3Handler, S3HandlerBridgeError};

#[derive(Debug)]
pub enum HandlerError {
    Bucket(BucketError),
    Object(ObjectError),
    Handler(HandlerOnlyError),
}

#[derive(Debug)]
pub enum BucketError {
    NotFound(String),
    AlreadyExists(String),
    NotEmpty(String),
    Internal(String),
}

#[derive(Debug)]
pub enum ObjectError {
    NotFound(String),
    UploadNotFound(String),
    Internal(String),
}

#[derive(Debug)]
pub enum HandlerOnlyError {
    BadRequest(String),
    MethodNotAllowed(String),
    Internal(String),
}

impl HandlerError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Handler(HandlerOnlyError::Internal(msg.into()))
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::Handler(HandlerOnlyError::BadRequest(msg.into()))
    }
    pub fn method_not_allowed(msg: impl Into<String>) -> Self {
        Self::Handler(HandlerOnlyError::MethodNotAllowed(msg.into()))
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bucket(e) => write!(f, "{e:?}"),
            Self::Object(e) => write!(f, "{e:?}"),
            Self::Handler(e) => write!(f, "{e:?}"),
        }
    }
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        use axum::http::StatusCode;
        let (status, code, msg) = match self {
            Self::Bucket(BucketError::NotFound(m)) => (StatusCode::NOT_FOUND, "NoSuchBucket", m),
            Self::Bucket(BucketError::AlreadyExists(m)) => (StatusCode::CONFLICT, "BucketAlreadyOwnedByYou", m),
            Self::Bucket(BucketError::NotEmpty(m)) => (StatusCode::CONFLICT, "BucketNotEmpty", m),
            Self::Bucket(BucketError::Internal(m)) => (StatusCode::INTERNAL_SERVER_ERROR, "InternalError", m),
            Self::Object(ObjectError::NotFound(m)) => (StatusCode::NOT_FOUND, "NoSuchKey", m),
            Self::Object(ObjectError::UploadNotFound(m)) => (StatusCode::NOT_FOUND, "NoSuchUpload", m),
            Self::Object(ObjectError::Internal(m)) => (StatusCode::INTERNAL_SERVER_ERROR, "InternalError", m),
            Self::Handler(HandlerOnlyError::BadRequest(m)) => (StatusCode::BAD_REQUEST, "BadRequest", m),
            Self::Handler(HandlerOnlyError::MethodNotAllowed(m)) => (StatusCode::METHOD_NOT_ALLOWED, "MethodNotAllowed", m),
            Self::Handler(HandlerOnlyError::Internal(m)) => (StatusCode::INTERNAL_SERVER_ERROR, "InternalError", m),
        };
        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><Error><Code>{}</Code><Message>{}</Message></Error>"#,
            code,
            msg.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        );
        (status, [("content-type", "application/xml")], body).into_response()
    }
}

pub fn router<T, E>(handler: T) -> Router
where
    T: S3Handler<E> + Send + Sync + 'static,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    let state = Arc::new(handler);
    Router::new()
        .merge(http_root::routes::<T, E>(state.clone()))
        .merge(http_bucket::routes::<T, E>(state.clone()))
        .merge(http_object::routes::<T, E>(state))
}
