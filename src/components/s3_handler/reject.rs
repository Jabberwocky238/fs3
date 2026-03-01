use crate::types::s3::request::{
    RejectedBucketApiRequest, RejectedObjectAclDeleteRequest, RejectedObjectTorrentRequest,
};
use crate::types::s3::response::S3Response;
use crate::types::traits::s3_handler::RejectedS3Handler;

pub async fn rejected_object_torrent<T, E>(
    handler: &T,
    req: RejectedObjectTorrentRequest,
) -> Result<S3Response, E>
where
    T: RejectedS3Handler<Error = E> + Send + Sync,
    E: Send + Sync + 'static,
{
    Ok(S3Response::RejectedApi(
        handler.rejected_object_torrent(req).await?,
    ))
}

pub async fn rejected_object_acl_delete<T, E>(
    handler: &T,
    req: RejectedObjectAclDeleteRequest,
) -> Result<S3Response, E>
where
    T: RejectedS3Handler<Error = E> + Send + Sync,
    E: Send + Sync + 'static,
{
    Ok(S3Response::RejectedApi(
        handler.rejected_object_acl_delete(req).await?,
    ))
}

pub async fn rejected_bucket_api<T, E>(
    handler: &T,
    req: RejectedBucketApiRequest,
) -> Result<S3Response, E>
where
    T: RejectedS3Handler<Error = E> + Send + Sync,
    E: Send + Sync + 'static,
{
    Ok(S3Response::RejectedApi(handler.rejected_bucket_api(req).await?))
}
