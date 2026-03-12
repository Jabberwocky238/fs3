use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::Method;
use axum::routing::{any, get};
use axum::Router;

use crate::types::s3::request::{ListBucketsDoubleSlashRequest, ListBucketsRequest, RootListenNotificationRequest};
use crate::types::s3::response::S3Response;
use crate::types::traits::s3_handler::S3Handler;

use super::util::{event_filter, has};
use super::{HandlerError};

pub fn routes<T>(state: Arc<T>) -> Router
where
    T: S3Handler + Send + Sync + 'static,
{
    Router::new()
        .route("/", any(root_entry::<T>))
        .route("//", get(root_double_slash::<T>))
        .with_state(state)
}

async fn root_double_slash<T>(State(handler): State<Arc<T>>) -> Result<S3Response, HandlerError>
where
    T: S3Handler + Send + Sync,
{
    let v = handler
        .list_buckets_double_slash(ListBucketsDoubleSlashRequest)
        .await
        .map_err(|e| HandlerError::internal(e.to_string()))?;
    Ok(S3Response::ListBucketsDoubleSlash(v))
}

async fn root_entry<T>(
    State(handler): State<Arc<T>>,
    method: Method,
    Query(q): Query<HashMap<String, String>>,
) -> Result<S3Response, HandlerError>
where
    T: S3Handler + Send + Sync,
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
