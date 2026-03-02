use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::Method;
use axum::routing::{any, get};
use axum::{Json, Router};

use crate::types::s3::response::S3Response;
use crate::types::s3::request::{ListBucketsDoubleSlashRequest, ListBucketsRequest, RootListenNotificationRequest};
use crate::types::traits::s3_handler::RootS3Handler;

use super::{root, HandlerError, S3Handler};
use super::util::{event_filter, has};

pub fn routes<T, E>(state: Arc<T>) -> Router
where
    T: S3Handler + RootS3Handler<Error = E> + Send + Sync + 'static,
    E: Display + Send + Sync + 'static,
{
    Router::new()
        .route("/", any(root_entry::<T, E>))
        .route("//", get(root_double_slash::<T, E>))
        .with_state(state)
}

async fn root_double_slash<T, E>(State(handler): State<Arc<T>>) -> Result<Json<S3Response>, HandlerError>
where
    T: S3Handler + RootS3Handler<Error = E> + Send + Sync,
    E: Display + Send + Sync + 'static,
{
    let resp = root::list_buckets_double_slash::<T, E>(&*handler, ListBucketsDoubleSlashRequest)
        .await
        .map_err(|e| HandlerError::internal(e.to_string()))?;
    Ok(Json(resp))
}

async fn root_entry<T, E>(
    State(handler): State<Arc<T>>,
    method: Method,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<S3Response>, HandlerError>
where
    T: S3Handler + RootS3Handler<Error = E> + Send + Sync,
    E: Display + Send + Sync + 'static,
{
    if method != Method::GET {
        return Err(HandlerError::method_not_allowed("root only supports GET"));
    }

    let resp = if has(&q, "events") {
        root::root_listen_notification::<T, E>(&*handler, RootListenNotificationRequest {
            filter: event_filter(&q),
        })
        .await
    } else {
        root::list_buckets::<T, E>(&*handler, ListBucketsRequest).await
    }
    .map_err(|e| HandlerError::internal(e.to_string()))?;

    Ok(Json(resp))
}
