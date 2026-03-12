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


use crate::types::traits::s3_handler::S3Handler;

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
