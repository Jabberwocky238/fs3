mod handler;
mod http_bucket;
mod http_object;
mod http_root;
mod util;

use std::fmt::{Display, Formatter};
use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub use handler::S3AxumHandler;

use crate::types::traits::s3_handler::S3Handler;

#[derive(Debug)]
pub struct HandlerError {
    status: StatusCode,
    message: String,
}

impl HandlerError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    pub fn method_not_allowed(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::METHOD_NOT_ALLOWED,
            message: message.into(),
        }
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for HandlerError {}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}

pub fn router<T>(handler: T) -> Router
where
    T: S3Handler + Send + Sync + 'static,
{
    let state = Arc::new(handler);
    Router::new()
        .merge(http_root::routes::<T>(state.clone()))
        .merge(http_bucket::routes::<T>(state.clone()))
        .merge(http_object::routes::<T>(state))
        .layer(axum::extract::DefaultBodyLimit::max(5 * 1024 * 1024 * 1024))
}
