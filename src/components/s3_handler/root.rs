use crate::types::s3::request::{
    ListBucketsDoubleSlashRequest, ListBucketsRequest, RootListenNotificationRequest,
};
use crate::types::s3::response::S3Response;
use crate::types::traits::s3_handler::RootS3Handler;

pub async fn root_listen_notification<T, E>(
    handler: &T,
    req: RootListenNotificationRequest,
) -> Result<S3Response, E>
where
    T: RootS3Handler<Error = E> + Send + Sync,
    E: Send + Sync + 'static,
{
    Ok(S3Response::RootListenNotification(
        handler.root_listen_notification(req).await?,
    ))
}

pub async fn list_buckets<T, E>(handler: &T, req: ListBucketsRequest) -> Result<S3Response, E>
where
    T: RootS3Handler<Error = E> + Send + Sync,
    E: Send + Sync + 'static,
{
    Ok(S3Response::ListBuckets(handler.list_buckets(req).await?))
}

pub async fn list_buckets_double_slash<T, E>(
    handler: &T,
    req: ListBucketsDoubleSlashRequest,
) -> Result<S3Response, E>
where
    T: RootS3Handler<Error = E> + Send + Sync,
    E: Send + Sync + 'static,
{
    Ok(S3Response::ListBucketsDoubleSlash(
        handler.list_buckets_double_slash(req).await?,
    ))
}
