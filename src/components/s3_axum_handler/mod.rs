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
pub struct HandlerError {
    pub status: axum::http::StatusCode,
    pub message: String,
}

impl HandlerError {
    pub fn internal(msg: impl Into<String>) -> Self {
        let message = msg.into();
        let status = Self::infer_status(&message);
        Self { status, message }
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self { status: axum::http::StatusCode::BAD_REQUEST, message: msg.into() }
    }
    pub fn method_not_allowed(msg: impl Into<String>) -> Self {
        Self { status: axum::http::StatusCode::METHOD_NOT_ALLOWED, message: msg.into() }
    }
    fn infer_status(msg: &str) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        if msg.contains("not found") { StatusCode::NOT_FOUND }
        else if msg.contains("already exists") { StatusCode::CONFLICT }
        else if msg.contains("not empty") { StatusCode::CONFLICT }
        else { StatusCode::INTERNAL_SERVER_ERROR }
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.message) }
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        let code = match self.status {
            axum::http::StatusCode::BAD_REQUEST => "BadRequest",
            axum::http::StatusCode::METHOD_NOT_ALLOWED => "MethodNotAllowed",
            axum::http::StatusCode::NOT_FOUND => "NoSuchKey",
            axum::http::StatusCode::CONFLICT => "Conflict",
            _ => "InternalError",
        };
        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><Error><Code>{}</Code><Message>{}</Message></Error>"#,
            code,
            self.message.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        );
        (self.status, [("content-type", "application/xml")], body).into_response()
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
