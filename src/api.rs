use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::server::{MultipartUpload, S3Server, SignedToken};

pub fn router(state: S3Server) -> Router {
    Router::new()
        .route("/api/presign/upload", post(presign_upload))
        .route("/api/presign/download", post(presign_download))
        .route("/api/multipart/init", post(multipart_init))
        .route("/api/multipart/presign-part", post(multipart_presign_part))
        .route("/api/multipart/complete", post(multipart_complete))
        .with_state(state)
}

// ── request types ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct PresignReq {
    bucket: String,
    key: String,
    #[serde(default = "default_expires")]
    expires_seconds: i64,
}

#[derive(Deserialize)]
struct MultipartInitReq {
    bucket: String,
    key: String,
}

#[derive(Deserialize)]
struct PresignPartReq {
    upload_id: String,
    part_number: i32,
    #[serde(default = "default_expires")]
    expires_seconds: i64,
}

#[derive(Deserialize)]
struct CompleteReq {
    upload_id: String,
    parts: Vec<i32>,
}

fn default_expires() -> i64 {
    3600
}

// ── presign upload ─────────────────────────────────────────────────

async fn presign_upload(
    State(state): State<S3Server>,
    Json(req): Json<PresignReq>,
) -> Response {
    let token_id = Uuid::new_v4().to_string();
    let token = SignedToken {
        op: "upload".into(),
        bucket: req.bucket.clone(),
        key: req.key.clone(),
        upload_id: String::new(),
        part_number: 0,
        expires_at: Utc::now() + Duration::seconds(req.expires_seconds),
    };
    state.presigned.lock().await.insert(token_id.clone(), token);

    let url = format!(
        "{}/{}/{}?token={}",
        state.listen_outer, req.bucket, req.key, token_id
    );
    info!(%url, "presign upload created");
    (StatusCode::OK, Json(serde_json::json!({ "url": url }))).into_response()
}

// ── presign download ───────────────────────────────────────────────

async fn presign_download(
    State(state): State<S3Server>,
    Json(req): Json<PresignReq>,
) -> Response {
    let token_id = Uuid::new_v4().to_string();
    let token = SignedToken {
        op: "download".into(),
        bucket: req.bucket.clone(),
        key: req.key.clone(),
        upload_id: String::new(),
        part_number: 0,
        expires_at: Utc::now() + Duration::seconds(req.expires_seconds),
    };
    state.presigned.lock().await.insert(token_id.clone(), token);

    let url = format!(
        "{}/{}/{}?token={}",
        state.listen_outer, req.bucket, req.key, token_id
    );
    info!(%url, "presign download created");
    (StatusCode::OK, Json(serde_json::json!({ "url": url }))).into_response()
}

// ── multipart init ─────────────────────────────────────────────────

async fn multipart_init(
    State(state): State<S3Server>,
    Json(req): Json<MultipartInitReq>,
) -> Response {
    let upload_id = Uuid::new_v4().to_string();
    let dir = std::env::temp_dir().join(format!("fs3-mp-{upload_id}"));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response();
    }

    state.uploads.lock().await.insert(
        upload_id.clone(),
        MultipartUpload {
            bucket: req.bucket,
            key: req.key,
            dir,
        },
    );

    info!(%upload_id, "multipart upload initiated");
    (StatusCode::OK, Json(serde_json::json!({ "upload_id": upload_id }))).into_response()
}

// ── multipart presign-part ─────────────────────────────────────────

async fn multipart_presign_part(
    State(state): State<S3Server>,
    Json(req): Json<PresignPartReq>,
) -> Response {
    let uploads = state.uploads.lock().await;
    let upload = match uploads.get(&req.upload_id) {
        Some(u) => u,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "upload not found" })),
            )
                .into_response();
        }
    };

    let token_id = Uuid::new_v4().to_string();
    let token = SignedToken {
        op: "upload-part".into(),
        bucket: upload.bucket.clone(),
        key: upload.key.clone(),
        upload_id: req.upload_id.clone(),
        part_number: req.part_number,
        expires_at: Utc::now() + Duration::seconds(req.expires_seconds),
    };

    let url = format!(
        "{}/{}/{}?token={}",
        state.listen_outer, upload.bucket, upload.key, token_id
    );
    drop(uploads);

    state.presigned.lock().await.insert(token_id, token);
    info!(%url, part = req.part_number, "presign part created");
    (StatusCode::OK, Json(serde_json::json!({ "url": url }))).into_response()
}

// ── multipart complete ─────────────────────────────────────────────

async fn multipart_complete(
    State(state): State<S3Server>,
    Json(req): Json<CompleteReq>,
) -> Response {
    let upload = {
        let uploads = state.uploads.lock().await;
        match uploads.get(&req.upload_id) {
            Some(u) => u.clone(),
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "upload not found" })),
                )
                    .into_response();
            }
        }
    };

    // Concatenate parts in order
    let mut merged = Vec::new();
    for pn in &req.parts {
        let part_path = upload.dir.join(format!("{pn}.part"));
        match std::fs::read(&part_path) {
            Ok(data) => merged.extend_from_slice(&data),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": format!("part {} missing: {}", pn, e)
                    })),
                )
                    .into_response();
            }
        }
    }

    // Write final object
    if let Err(e) = state.mounts.put(&upload.bucket, &upload.key, &merged) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response();
    }

    // Cleanup temp dir and upload entry
    let _ = std::fs::remove_dir_all(&upload.dir);
    state.uploads.lock().await.remove(&req.upload_id);

    info!(bucket = %upload.bucket, key = %upload.key, "multipart complete");
    (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response()
}
