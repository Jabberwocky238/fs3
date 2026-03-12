mod util;
mod http_bucket;
mod http_object;
mod http_root;
mod handler;

use std::fmt::{Display, Formatter};
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Router;

pub use handler::S3AxumHandler;

use crate::types::traits::BoxError;
use crate::types::traits::s3_handler::{S3Handler, S3HandlerBridgeError};

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

fn bridge_handler_error(err: BoxError) -> HandlerError {
    if let Some(bridge) = err.downcast_ref::<S3HandlerBridgeError>() {
        return match bridge {
            S3HandlerBridgeError::Unsupported(message) => HandlerError::bad_request(*message),
            S3HandlerBridgeError::InvalidRequest(message) => HandlerError::bad_request(message.clone()),
            S3HandlerBridgeError::AccessDenied(message) => HandlerError {
                status: StatusCode::FORBIDDEN,
                message: message.clone(),
            },
            S3HandlerBridgeError::PreconditionFailed => HandlerError {
                status: StatusCode::PRECONDITION_FAILED,
                message: bridge.to_string(),
            },
            S3HandlerBridgeError::NotModified => HandlerError {
                status: StatusCode::NOT_MODIFIED,
                message: bridge.to_string(),
            },
            S3HandlerBridgeError::InvalidVersioningStatus(message) => HandlerError::bad_request(message.clone()),
            S3HandlerBridgeError::XmlParse(message) => HandlerError::bad_request(message.clone()),
        };
    }
    HandlerError::internal(err.to_string())
}

pub(crate) fn bucket_err(err: BoxError) -> HandlerError {
    bridge_handler_error(err)
}

pub(crate) fn object_err(err: BoxError) -> HandlerError {
    bridge_handler_error(err)
}
