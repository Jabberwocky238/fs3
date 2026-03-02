use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::Method;
use axum::routing::{any, get};
use axum::Router;

use crate::types::s3::request::{ListBucketsDoubleSlashRequest, ListBucketsRequest, RootListenNotificationRequest};
use crate::types::s3::response::S3Response;
use crate::types::errors::S3EngineError;
use crate::types::traits::s3_handler::{S3Handler, S3HandlerBridgeError};

use super::util::{event_filter, has};
use super::{HandlerError};

pub fn routes<T, E>(state: Arc<T>) -> Router
where
    T: S3Handler<E> + Send + Sync + 'static,
    E: std::fmt::Display + From<S3HandlerBridgeError> + From<S3EngineError> + 'static,
{
    Router::new()
        .route("/", any(root_entry::<T, E>))
        .route("//", get(root_double_slash::<T, E>))
        .with_state(state)
}

async fn root_double_slash<T, E>(State(handler): State<Arc<T>>) -> Result<S3Response, HandlerError>
where
    T: S3Handler<E> + Send + Sync,
    E: std::fmt::Display + From<S3HandlerBridgeError> + From<S3EngineError> + 'static,
{
    let v = handler
        .list_buckets_double_slash(ListBucketsDoubleSlashRequest)
        .await
        .map_err(|e| HandlerError::internal(e.to_string()))?;
    Ok(S3Response::ListBucketsDoubleSlash(v))
}

async fn root_entry<T, E>(
    State(handler): State<Arc<T>>,
    method: Method,
    Query(q): Query<HashMap<String, String>>,
) -> Result<S3Response, HandlerError>
where
    T: S3Handler<E> + Send + Sync,
    E: std::fmt::Display + From<S3HandlerBridgeError> + From<S3EngineError> + 'static,
{
    if method != Method::GET {
        return Err(HandlerError::method_not_allowed("root only supports GET"));
    }
    if has(&q, "events") {
        let v = handler
            .root_listen_notification(RootListenNotificationRequest { filter: event_filter(&q) })
            .await
            .map_err(|e| HandlerError::internal(e.to_string()))?;
        return Ok(S3Response::RootListenNotification(v));
    }
    let v = handler
        .list_buckets(ListBucketsRequest)
        .await
        .map_err(|e| HandlerError::internal(e.to_string()))?;
    Ok(S3Response::ListBuckets(v))
}
