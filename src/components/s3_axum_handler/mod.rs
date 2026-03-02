mod util;
mod http_bucket;
mod http_object;
mod http_root;

use std::fmt::{Display, Formatter};
use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use axum::{Json, Router};

use crate::types::traits::s3_handler::{
    BucketS3Handler, ObjectS3Handler, RejectedBucketS3Handler, RejectedObjectS3Handler,
    RootS3Handler, S3Handler,
};

#[derive(Debug)]
pub struct HandlerError {
    pub status: axum::http::StatusCode,
    pub message: String,
}

impl HandlerError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self { status: axum::http::StatusCode::INTERNAL_SERVER_ERROR, message: msg.into() }
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self { status: axum::http::StatusCode::BAD_REQUEST, message: msg.into() }
    }
    pub fn method_not_allowed(msg: impl Into<String>) -> Self {
        Self { status: axum::http::StatusCode::METHOD_NOT_ALLOWED, message: msg.into() }
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.message) }
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        (self.status, Json(serde_json::json!({ "error": self.message }))).into_response()
    }
}

pub fn router<T, E>(handler: T) -> Router
where
    T: S3Handler
        + ObjectS3Handler<Error = E>
        + BucketS3Handler<Error = E>
        + RootS3Handler<Error = E>
        + RejectedObjectS3Handler<Error = E>
        + RejectedBucketS3Handler<Error = E>
        + Send
        + Sync
        + 'static,
    E: Display + Send + Sync + 'static,
{
    let state = Arc::new(handler);
    Router::new()
        .merge(http_root::routes::<T, E>(state.clone()))
        .merge(http_bucket::routes::<T, E>(state.clone()))
        .merge(http_object::routes::<T, E>(state))
}
