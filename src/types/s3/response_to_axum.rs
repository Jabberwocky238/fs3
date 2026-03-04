use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use super::response::S3Response;
use super::response_to_xml::s3_response_to_xml;

impl IntoResponse for S3Response {
    fn into_response(self) -> Response {
        match self {
            // Empty 200
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
            | S3Response::PostRestoreObject(_) => StatusCode::OK.into_response(),

            S3Response::HeadBucket(r) => {
                let sc = StatusCode::from_u16(r.meta.status_code).unwrap_or(StatusCode::OK);
                (sc, "").into_response()
            }

            // Empty 204
            S3Response::DeleteBucket(_)
            | S3Response::DeleteBucketPolicy(_)
            | S3Response::DeleteBucketTagging(_)
            | S3Response::DeleteBucketEncryption(_)
            | S3Response::DeleteBucketLifecycle(_)
            | S3Response::DeleteBucketReplication(_)
            | S3Response::DeleteBucketWebsite(_)
            | S3Response::DeleteBucketCors(_)
            | S3Response::DeleteObject(_)
            | S3Response::DeleteObjectTagging(_) => StatusCode::NO_CONTENT.into_response(),

            // Streaming body
            S3Response::GetObject(r) => {
                let mut resp = axum::body::Body::from_stream(r.body).into_response();
                if let Some(size) = r.size {
                    if let Ok(v) = size.to_string().parse() {
                        resp.headers_mut().insert("content-length", v);
                    }
                }
                if let Some(etag) = &r.meta.etag {
                    if let Ok(v) = format!("\"{}\"", etag).parse() {
                        resp.headers_mut().insert("etag", v);
                    }
                }
                resp
            }

            S3Response::GetObjectLambda(r) => {
                let sc = StatusCode::from_u16(r.meta.status_code).unwrap_or(StatusCode::OK);
                (sc, r.body).into_response()
            }

            S3Response::SelectObjectContent(r) => (StatusCode::OK, r.payload).into_response(),

            // JSON response
            S3Response::GetBucketPolicy(r) => {
                (StatusCode::OK, [("content-type", "application/json")], r.config.clone()).into_response()
            }

            // XML responses
            _ => {
                if let Some(xml) = s3_response_to_xml(&self) {
                    (StatusCode::OK, [("content-type", "application/xml")], xml).into_response()
                } else {
                    StatusCode::OK.into_response()
                }
            }
        }
    }
}
