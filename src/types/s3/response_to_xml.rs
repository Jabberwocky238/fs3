use super::response::*;
use super::response_to_xml_impl::XMLResponse;

impl From<&S3Response> for Option<XMLResponse> {
    fn from(resp: &S3Response) -> Self {
    match resp {
        // Empty responses (no XML body)
        S3Response::PutBucket(_)
        | S3Response::PutBucketPolicy(_)
        | S3Response::PutBucketTagging(_)
        | S3Response::PutBucketVersioning(_)
        | S3Response::PutBucketNotification(_)
        | S3Response::PutBucketLifecycle(_)
        | S3Response::PutBucketReplicationConfig(_)
        | S3Response::PutBucketEncryption(_)
        | S3Response::PutBucketObjectLockConfig(_)
        | S3Response::PutBucketAcl(_)
        | S3Response::PutBucketCors(_)
        | S3Response::PutObjectAcl(_)
        | S3Response::PutObjectTagging(_)
        | S3Response::PutObjectRetention(_)
        | S3Response::PutObjectLegalHold(_)
        | S3Response::ResetBucketReplicationStart(_)
        | S3Response::ResetBucketReplicationStatus(_)
        | S3Response::PostRestoreObject(_)
        | S3Response::HeadBucket(_)
        | S3Response::DeleteBucket(_)
        | S3Response::DeleteBucketPolicy(_)
        | S3Response::DeleteBucketTagging(_)
        | S3Response::DeleteBucketEncryption(_)
        | S3Response::DeleteBucketLifecycle(_)
        | S3Response::DeleteBucketReplication(_)
        | S3Response::DeleteBucketWebsite(_)
        | S3Response::DeleteBucketCors(_)
        | S3Response::DeleteObject(_)
        | S3Response::DeleteObjectTagging(_) => None,

        S3Response::GetBucketLocation(r) => Some(r.into()),
        S3Response::ListBuckets(r) => Some(r.into()),
        S3Response::ListBucketsDoubleSlash(r) => Some(r.into()),
        S3Response::ListObjectsV1(r) => Some(r.into()),
        S3Response::ListObjectsV2(r) => Some(r.into()),
        S3Response::ListObjectsV2M(r) => Some(r.into()),
        S3Response::ListObjectVersions(r) => Some(r.into()),
        S3Response::ListObjectVersionsM(r) => Some(r.into()),
        S3Response::ListMultipartUploads(r) => Some(r.into()),
        S3Response::ListObjectParts(r) => Some(r.into()),
        S3Response::NewMultipartUpload(r) => Some(r.into()),
        S3Response::AbortMultipartUpload(r) => Some(r.into()),
        S3Response::CompleteMultipartUpload(r) => Some(r.into()),

        S3Response::GetBucketLifecycle(_) => None,
        S3Response::GetBucketEncryption(_) => None,
        S3Response::GetBucketObjectLockConfig(_) => None,
        S3Response::GetBucketReplicationConfig(_) => None,
        S3Response::GetBucketVersioning(r) => Some(r.into()),
        S3Response::GetBucketNotification(r) => Some(r.into()),
        S3Response::GetBucketAcl(_) => None,
        S3Response::GetBucketCors(_) => None,
        S3Response::GetBucketWebsite(_) => None,
        S3Response::GetBucketAccelerate(_) => None,
        S3Response::GetBucketRequestPayment(_) => None,
        S3Response::GetBucketLogging(_) => None,
        S3Response::GetBucketTagging(r) => Some(r.into()),
        S3Response::GetObjectAcl(_) => None,
        S3Response::GetObjectTagging(r) => Some(r.into()),
        S3Response::GetObjectRetention(_) => None,
        S3Response::GetObjectLegalHold(_) => None,

        S3Response::CopyObject(r) => Some(r.into()),
        S3Response::PutObject(r) => Some(r.into()),
        S3Response::HeadObject(_) => None,
        S3Response::GetObject(_) => None,
        S3Response::GetObjectLambda(_) => None,
        S3Response::SelectObjectContent(_) => None,
        S3Response::GetBucketPolicy(_) => None,

        _ => None,
    }
}
}

