use std::io::Read;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{any, get};
use axum::Router;
use chrono::Utc;
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::authentic;
use crate::mount::{MountError, ObjectInfo};
use crate::server::S3Server;
#[cfg(feature = "multi-user")]
use crate::storage::UserStore;
use crate::server::SignedToken;

pub fn router(state: S3Server) -> Router {
    Router::new()
        .route("/", get(list_buckets).head(not_supported))
        .route("/{*path}", any(handle_any))
        .with_state(state)
}

async fn not_supported() -> impl IntoResponse {
    (StatusCode::METHOD_NOT_ALLOWED, "method not allowed")
}

async fn list_buckets(
    State(state): State<S3Server>,
    headers: HeaderMap,
    Query(q): Query<S3Query>,
) -> Response {
    if let Err(e) = authorize(&state, &headers, &q).await {
        return s3_error(e.0, e.1, e.2);
    }

    let mut buckets_xml = String::new();
    for b in state.mounts.buckets() {
        buckets_xml.push_str(&format!(
            "<Bucket><Name>{}</Name><CreationDate>{}</CreationDate></Bucket>",
            xml_escape(&b),
            Utc::now().to_rfc3339()
        ));
    }
    let body = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ListAllMyBucketsResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"><Owner><ID>local</ID><DisplayName>local</DisplayName></Owner><Buckets>{}</Buckets></ListAllMyBucketsResult>",
        buckets_xml
    );
    (
        StatusCode::OK,
        [("content-type", "application/xml")],
        Html(body),
    )
        .into_response()
}

async fn handle_any(
    State(state): State<S3Server>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    Query(q): Query<S3Query>,
    body: Bytes,
) -> Response {
    debug!(%method, %path, "s3 request");

    // Presigned token path — bypass normal auth
    if let Some(ref token_id) = q.token {
        return handle_presigned_request(&state, token_id, method, body).await;
    }

    if let Err(e) = authorize(&state, &headers, &q).await {
        warn!(%method, %path, code = e.1, "auth rejected");
        return s3_error(e.0, e.1, e.2);
    }

    let path = path.trim_start_matches('/');
    let mut iter = path.splitn(2, '/');
    let bucket = iter.next().unwrap_or_default().to_string();
    let key = iter.next().unwrap_or_default().to_string();

    if bucket.is_empty() {
        return s3_error(
            StatusCode::NOT_IMPLEMENTED,
            "NotImplemented",
            "Only path-style S3 requests are supported",
        );
    }

    if key.is_empty() {
        return handle_bucket(&state, method, bucket, q).await;
    }

    handle_object(&state, method, bucket, key, body, headers).await
}

async fn handle_bucket(state: &S3Server, method: Method, bucket: String, q: S3Query) -> Response {
    // Single-bucket mode: reject create/delete
    if !state.cfg.multi_bucket_enabled && matches!(method, Method::PUT | Method::DELETE) {
        return s3_error(
            StatusCode::FORBIDDEN,
            "AccessDenied",
            "Bucket creation and deletion are disabled in single-bucket mode",
        );
    }

    match method {
        Method::HEAD => {
            if state.mounts.has_bucket(&bucket) {
                StatusCode::OK.into_response()
            } else {
                s3_error(
                    StatusCode::NOT_FOUND,
                    "NoSuchBucket",
                    "The specified bucket does not exist",
                )
            }
        }
        Method::PUT => match state.mounts.ensure_bucket(&bucket) {
            Ok(_) => {
                info!(%bucket, "bucket created/ensured");
                StatusCode::OK.into_response()
            }
            Err(MountError::NoSuchBucket) => s3_error(
                StatusCode::NOT_FOUND,
                "NoSuchBucket",
                "Bucket is not mounted in config",
            ),
            Err(e) => s3_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &e.to_string(),
            ),
        },
        Method::GET => {
            if q.location.is_some() {
                let body = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><LocationConstraint xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"></LocationConstraint>";
                return (
                    StatusCode::OK,
                    [("content-type", "application/xml")],
                    Html(body.to_string()),
                )
                    .into_response();
            }
            if q.list_type.as_deref() == Some("2") || q.is_empty() {
                return list_objects_v2(state, &bucket, q);
            }
            s3_error(
                StatusCode::NOT_IMPLEMENTED,
                "NotImplemented",
                "Bucket query operation is not supported",
            )
        }
        _ => s3_error(
            StatusCode::METHOD_NOT_ALLOWED,
            "MethodNotAllowed",
            "Method not allowed",
        ),
    }
}

async fn handle_object(
    state: &S3Server,
    method: Method,
    bucket: String,
    key: String,
    body: Bytes,
    headers: HeaderMap,
) -> Response {
    match method {
        Method::GET => match state.mounts.open(&bucket, &key) {
            Ok((mut f, obj)) => {
                let mut buf = Vec::new();
                if let Err(e) = f.read_to_end(&mut buf) {
                    return s3_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "InternalError",
                        &e.to_string(),
                    );
                }
                let mut resp = (StatusCode::OK, buf.clone()).into_response();
                let headers = resp.headers_mut();
                headers.insert("etag", format!("\"{}\"", etag_bytes(&buf)).parse().unwrap());
                headers.insert("content-length", obj.size.to_string().parse().unwrap());
                headers.insert("accept-ranges", "bytes".parse().unwrap());
                resp
            }
            Err(e) => mount_to_s3_error(e),
        },
        Method::HEAD => match state.mounts.stat(&bucket, &key) {
            Ok(obj) => {
                let etag = etag_path(&obj);
                let mut resp = StatusCode::OK.into_response();
                let headers = resp.headers_mut();
                headers.insert("etag", format!("\"{}\"", etag).parse().unwrap());
                headers.insert("content-length", obj.size.to_string().parse().unwrap());
                headers.insert("accept-ranges", "bytes".parse().unwrap());
                resp
            }
            Err(e) => mount_to_s3_error(e),
        },
        Method::PUT => {
            let payload = if is_aws_chunked(&headers) {
                match decode_aws_chunked(&body) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(%bucket, %key, error = %e, "aws-chunked decode failed");
                        return s3_error(StatusCode::BAD_REQUEST, "InvalidRequest", &e);
                    }
                }
            } else {
                body.to_vec()
            };
            match state.mounts.put(&bucket, &key, &payload) {
                Ok(_) => {
                    debug!(%bucket, %key, size = payload.len(), "object put");
                    (
                        StatusCode::OK,
                        [("etag", &format!("\"{}\"", etag_bytes(&payload)))],
                    )
                        .into_response()
                }
                Err(e) => {
                    warn!(%bucket, %key, error = %e, "put failed");
                    mount_to_s3_error(e)
                }
            }
        }
        Method::DELETE => match state.mounts.delete(&bucket, &key) {
            Ok(_) => {
                debug!(%bucket, %key, "object deleted");
                StatusCode::NO_CONTENT.into_response()
            }
            Err(MountError::NoSuchKey) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => {
                warn!(%bucket, %key, error = %e, "delete failed");
                mount_to_s3_error(e)
            }
        },
        _ => s3_error(
            StatusCode::METHOD_NOT_ALLOWED,
            "MethodNotAllowed",
            "Method not allowed",
        ),
    }
}

fn list_objects_v2(state: &S3Server, bucket: &str, q: S3Query) -> Response {
    let prefix = q.prefix.unwrap_or_default();
    let delimiter = q.delimiter.unwrap_or_default();
    let token = q.continuation_token.unwrap_or_default();
    let max_keys = q.max_keys.unwrap_or(1000);

    let lr = match state
        .mounts
        .list(bucket, &prefix, &delimiter, &token, max_keys)
    {
        Ok(v) => v,
        Err(MountError::NoSuchBucket) => {
            return s3_error(
                StatusCode::NOT_FOUND,
                "NoSuchBucket",
                "The specified bucket does not exist",
            );
        }
        Err(e) => {
            return s3_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                &e.to_string(),
            );
        }
    };

    let mut contents = String::new();
    for obj in lr.keys {
        contents.push_str(&format!(
            "<Contents><Key>{}</Key><LastModified>{}</LastModified><ETag>\"{}\"</ETag><Size>{}</Size><StorageClass>STANDARD</StorageClass></Contents>",
            xml_escape(&obj.key),
            obj.last_modified.to_rfc3339(),
            etag_path(&obj),
            obj.size
        ));
    }
    let mut cps = String::new();
    for cp in lr.common_prefixes {
        cps.push_str(&format!(
            "<CommonPrefixes><Prefix>{}</Prefix></CommonPrefixes>",
            xml_escape(&cp)
        ));
    }

    let body = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ListBucketResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"><Name>{}</Name><Prefix>{}</Prefix><Delimiter>{}</Delimiter><MaxKeys>{}</MaxKeys><IsTruncated>{}</IsTruncated><ContinuationToken>{}</ContinuationToken><NextContinuationToken>{}</NextContinuationToken>{}{}</ListBucketResult>",
        xml_escape(bucket),
        xml_escape(&prefix),
        xml_escape(&delimiter),
        max_keys,
        if lr.truncated { "true" } else { "false" },
        xml_escape(&token),
        xml_escape(&lr.next_token),
        contents,
        cps
    );

    (
        StatusCode::OK,
        [("content-type", "application/xml")],
        Html(body),
    )
        .into_response()
}

#[derive(Debug, Deserialize, Clone, Default)]
struct S3Query {
    #[serde(rename = "list-type")]
    list_type: Option<String>,
    prefix: Option<String>,
    delimiter: Option<String>,
    #[serde(rename = "continuation-token")]
    continuation_token: Option<String>,
    #[serde(rename = "max-keys")]
    max_keys: Option<usize>,
    location: Option<String>,
    #[serde(rename = "X-Amz-Credential")]
    x_amz_credential: Option<String>,
    token: Option<String>,
}

impl S3Query {
    fn is_empty(&self) -> bool {
        self.list_type.is_none()
            && self.prefix.is_none()
            && self.delimiter.is_none()
            && self.continuation_token.is_none()
            && self.max_keys.is_none()
            && self.location.is_none()
            && self.x_amz_credential.is_none()
            && self.token.is_none()
    }
}

async fn authorize(
    state: &S3Server,
    headers: &HeaderMap,
    q: &S3Query,
) -> Result<(), (StatusCode, &'static str, &'static str)> {
    #[cfg(feature = "multi-user")]
    let users = {
        let raw = state.store.list_users().await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                "load users failed",
            )
        })?;
        raw.into_iter()
            .map(|u| authentic::AuthUser {
                user_id: u.user_id,
                enabled: u.enabled,
                access_key: u.access_key,
                secret_key: u.secret_key,
            })
            .collect::<Vec<_>>()
    };

    let query_credential = q.x_amz_credential.as_ref().map(|v| url_decode(v));
    #[cfg(feature = "multi-user")]
    {
        match authentic::check_access_key(&users, headers, query_credential.as_deref()) {
            Ok(_) => Ok(()),
            Err(_) => Err((StatusCode::FORBIDDEN, "AccessDenied", "Access denied")),
        }
    }
    #[cfg(not(feature = "multi-user"))]
    {
        let _ = (state, headers, query_credential);
        Ok(())
    }
}

fn url_decode(v: &str) -> String {
    let mut out = String::new();
    let bytes = v.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h = &v[i + 1..i + 3];
            if let Ok(x) = u8::from_str_radix(h, 16) {
                out.push(x as char);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(' ');
        } else {
            out.push(bytes[i] as char);
        }
        i += 1;
    }
    out
}

fn s3_error(status: StatusCode, code: &str, message: &str) -> Response {
    let body = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Error><Code>{}</Code><Message>{}</Message></Error>",
        xml_escape(code),
        xml_escape(message)
    );
    (status, [("content-type", "application/xml")], Html(body)).into_response()
}

fn mount_to_s3_error(err: MountError) -> Response {
    match err {
        MountError::NoSuchBucket => s3_error(
            StatusCode::NOT_FOUND,
            "NoSuchBucket",
            "The specified bucket does not exist",
        ),
        MountError::NoSuchKey => s3_error(
            StatusCode::NOT_FOUND,
            "NoSuchKey",
            "The specified key does not exist",
        ),
        MountError::ReadOnly => {
            s3_error(StatusCode::FORBIDDEN, "AccessDenied", "Bucket is read-only")
        }
        MountError::BadKey => s3_error(
            StatusCode::BAD_REQUEST,
            "InvalidObjectName",
            "Invalid object key",
        ),
        MountError::Io(e) => s3_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "InternalError",
            &e.to_string(),
        ),
        MountError::Config(e) => s3_error(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &e),
    }
}

fn etag_path(obj: &ObjectInfo) -> String {
    match std::fs::read(&obj.physical_path) {
        Ok(v) => etag_bytes(&v),
        Err(_) => String::new(),
    }
}

fn etag_bytes(data: &[u8]) -> String {
    format!("{:x}", md5::compute(data))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn is_aws_chunked(headers: &HeaderMap) -> bool {
    let ce = headers
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    let sha = headers
        .get("x-amz-content-sha256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    ce.contains("aws-chunked") || sha.starts_with("STREAMING-AWS4-HMAC-SHA256")
}

fn decode_aws_chunked(body: &[u8]) -> Result<Vec<u8>, String> {
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < body.len() {
        let line_end =
            find_crlf(body, i).ok_or_else(|| "malformed aws-chunked stream".to_string())?;
        let line = std::str::from_utf8(&body[i..line_end]).map_err(|e| e.to_string())?;
        i = line_end + 2;
        if line.trim().is_empty() {
            continue;
        }
        let size_hex = line.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_hex, 16).map_err(|e| e.to_string())?;
        if size == 0 {
            break;
        }
        if i + size > body.len() {
            return Err("chunk out of bounds".into());
        }
        out.extend_from_slice(&body[i..i + size]);
        i += size;
        if i + 1 >= body.len() || &body[i..i + 2] != b"\r\n" {
            return Err("chunk missing CRLF".into());
        }
        i += 2;
    }
    Ok(out)
}

fn find_crlf(b: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    while i + 1 < b.len() {
        if b[i] == b'\r' && b[i + 1] == b'\n' {
            return Some(i);
        }
        i += 1;
    }
    None
}

async fn handle_presigned_request(
    state: &S3Server,
    token_id: &str,
    method: Method,
    body: Bytes,
) -> Response {
    let token = {
        let map = state.presigned.lock().await;
        match map.get(token_id) {
            Some(t) => t.clone(),
            None => return s3_error(StatusCode::FORBIDDEN, "AccessDenied", "Invalid presign token"),
        }
    };

    if Utc::now() > token.expires_at {
        state.presigned.lock().await.remove(token_id);
        return s3_error(StatusCode::FORBIDDEN, "AccessDenied", "Presign token expired");
    }

    match token.op.as_str() {
        "upload" => {
            if method != Method::PUT {
                return s3_error(StatusCode::METHOD_NOT_ALLOWED, "MethodNotAllowed", "Expected PUT");
            }
            let _ = state.mounts.ensure_bucket(&token.bucket);
            match state.mounts.put(&token.bucket, &token.key, &body) {
                Ok(_) => {
                    state.presigned.lock().await.remove(token_id);
                    let etag = etag_bytes(&body);
                    (StatusCode::OK, [("etag", format!("\"{}\"", etag))]).into_response()
                }
                Err(e) => mount_to_s3_error(e),
            }
        }
        "download" => {
            if method != Method::GET {
                return s3_error(StatusCode::METHOD_NOT_ALLOWED, "MethodNotAllowed", "Expected GET");
            }
            match state.mounts.open(&token.bucket, &token.key) {
                Ok((mut f, _obj)) => {
                    let mut buf = Vec::new();
                    if let Err(e) = f.read_to_end(&mut buf) {
                        return s3_error(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &e.to_string());
                    }
                    (StatusCode::OK, buf).into_response()
                }
                Err(e) => mount_to_s3_error(e),
            }
        }
        "upload-part" => handle_presigned_part(state, &token, token_id, method, body).await,
        _ => s3_error(StatusCode::BAD_REQUEST, "InvalidRequest", "Unknown presign op"),
    }
}

async fn handle_presigned_part(
    state: &S3Server,
    token: &SignedToken,
    token_id: &str,
    method: Method,
    body: Bytes,
) -> Response {
    if method != Method::PUT {
        return s3_error(StatusCode::METHOD_NOT_ALLOWED, "MethodNotAllowed", "Expected PUT");
    }

    let uploads = state.uploads.lock().await;
    let upload = match uploads.get(&token.upload_id) {
        Some(u) => u,
        None => return s3_error(StatusCode::NOT_FOUND, "NoSuchUpload", "Upload not found"),
    };

    let part_path = upload.dir.join(format!("{}.part", token.part_number));
    drop(uploads);

    if let Err(e) = std::fs::write(&part_path, &body) {
        return s3_error(StatusCode::INTERNAL_SERVER_ERROR, "InternalError", &e.to_string());
    }

    state.presigned.lock().await.remove(token_id);
    let etag = etag_bytes(&body);
    (StatusCode::OK, [("etag", format!("\"{}\"", etag))]).into_response()
}
