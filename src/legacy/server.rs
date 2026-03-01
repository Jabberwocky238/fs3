use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::Router;
use axum::response::Response;
use chrono::{DateTime, Utc};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};
use crate::policy::Policy;
use crate::config::Config;
use crate::mount::{MountManager, new_mount};
use crate::policy::PolicyEngine;

use crate::storage::PolicyStore;
use crate::storage::factory::{self, StorageBackend};
use crate::storage::types::StorageSnapshot;
use crate::storage::types::UserRecord;

// ── S3Server ────────────────────────────────────────────────────────────

const DEFAULT_BUCKET: &str = "default";

#[derive(Debug, Clone)]
pub struct SignedToken {
    pub op: String,
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub part_number: i32,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MultipartUpload {
    pub bucket: String,
    pub key: String,
    pub dir: std::path::PathBuf,
}

#[derive(Clone)]
pub struct S3Server {
    pub mounts: Arc<dyn MountManager>,
    pub policy: Arc<RwLock<PolicyEngine>>,
    pub store: Arc<StorageBackend>,
    pub cfg: Config,
    pub listen_inner: String,
    pub listen_outer: String,
    pub presigned: Arc<Mutex<HashMap<String, SignedToken>>>,
    pub uploads: Arc<Mutex<HashMap<String, MultipartUpload>>>,
}

pub async fn run_with_config_path(path: &str) -> Result<()> {
    S3Server::from_config_path(path).await?.serve().await
}

pub async fn run(cfg: Config) -> Result<()> {
    S3Server::from_config(cfg).await?.serve().await
}

impl S3Server {
    pub async fn from_config_path(path: &str) -> Result<Self> {
        let cfg = load_config(path)?;
        Self::from_config(cfg).await
    }

    pub async fn from_config(cfg: Config) -> Result<Self> {
        info!(mode = ?cfg.mount.mode, path = %cfg.mount.path, "initializing mounts");
        let mounts: Arc<dyn MountManager> =
            Arc::from(new_mount(&cfg.mount).context("init mounts")?);

        if !cfg.multi_bucket_enabled {
            mounts
                .ensure_bucket(DEFAULT_BUCKET)
                .context("ensure default bucket")?;
            info!("single bucket mode, default bucket ensured");
        }
        
        let seed = Self::prepare_init_storage(cfg);
        info!(kind = ?cfg.storage.kind, "initializing storage backend");
        let store = factory::new_store(&cfg.storage, seed)
            .await
            .context("init metadata store")?;

        let groups = store
            .list_policy_groups()
            .await
            .context("load policy groups")?;

        let listen_inner = normalize_outer(&cfg.listen_inner);
        let listen_outer = normalize_outer(&cfg.listen_outer);
        Ok(S3Server {
            mounts,
            policy: Arc::new(RwLock::new(PolicyEngine::new(groups))),
            store,
            cfg,
            listen_inner,
            listen_outer,
            presigned: Arc::new(Mutex::new(Default::default())),
            uploads: Arc::new(Mutex::new(Default::default())),
        })
    }

    fn prepare_init_storage(cfg: &Config) -> StorageSnapshot {
        let admin_sk = std::env::var("FS3_ADMIN_SECRET_KEY").unwrap_or_else(|_| "fs3_admin-sk".into());
        let admin_ak = std::env::var("FS3_ADMIN_ACCESS_KEY").unwrap_or_else(|_| "fs3_admin-ak".into());
        let admin_user = UserRecord {
            user_id: "fs3_admin".into(),
            enabled: true,
            access_key: admin_ak,
            secret_key: admin_sk,
            groups: vec!["fs3_admin".into()],
            attrs: std::collections::HashMap::new(),
        };
        let policys = if cfg.multi_bucket_enabled {
            vec![
                crate::policy::Policy {
                            bucket: "*".into(),
                            key_prefix: "*".into(),
                            allow_read: true,
                            allow_write: true,
                        }
            ]
        } else {
            vec![]
        }

        let seed = StorageSnapshot {
            users: vec![admin_user],
            policies: policys,
            bucket_metadata: vec![],
        };
        seed
    }

    pub async fn serve(self) -> Result<()> {
        let ctrl_router: Router = ctrl_gw_router(self.clone());
        let open_router: Router = open_gw_router(self.clone());

        let ctrl_addr = normalize_listen(&self.cfg.listen_inner);
        let open_addr = normalize_listen(&self.cfg.listen_outer);

        let ctrl_listener = TcpListener::bind(&ctrl_addr).await?;
        let open_listener = TcpListener::bind(&open_addr).await?;

        info!("control gateway listening on {}", ctrl_addr);
        info!("open gateway listening on {}", open_addr);

        let ctrl_task = tokio::spawn(async move { axum::serve(ctrl_listener, ctrl_router).await });
        let open_task = tokio::spawn(async move { axum::serve(open_listener, open_router).await });

        tokio::select! {
            res = ctrl_task => { res??; }
            res = open_task => { res??; }
        }
        Ok(())
    }
}

// ── helpers ────────────────────────────────────────────────────────────

pub fn load_config(path: &str) -> Result<Config> {
    let data =
        std::fs::read_to_string(path).with_context(|| format!("reading config file: {}", path))?;
    serde_json::from_str(&data).context("parsing config JSON")
}

fn normalize_listen(addr: &str) -> String {
    if addr.starts_with(':') {
        format!("0.0.0.0{}", addr)
    } else {
        addr.to_string()
    }
}

fn normalize_outer(addr: &str) -> String {
    if addr.is_empty() {
        "http://localhost:3001".to_string()
    } else {
        let s = addr.trim_end_matches('/');
        if s.starts_with("http://") || s.starts_with("https://") {
            s.to_string()
        } else {
            format!("http://{s}")
        }
    }
}

fn ctrl_gw_router(state: S3Server) -> Router {
    use axum::routing::get;
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .merge(crate::api::router(state.clone()))
        .merge(crate::s3::router(state))
}

fn open_gw_router(state: S3Server) -> Router {
    use axum::extract::{Path, State};
    use axum::response::IntoResponse;
    use axum::routing::get;

    let bucket_on = state.cfg.multi_bucket_enabled;

    async fn h_user_bucket(
        State(s): State<S3Server>,
        Path((user, bucket, obj)): Path<(String, String, String)>,
    ) -> impl IntoResponse {
        serve_open_gw(&s, &user, &bucket, &obj).await
    }

    async fn h_user(
        State(s): State<S3Server>,
        Path((user, obj)): Path<(String, String)>,
    ) -> impl IntoResponse {
        serve_open_gw(&s, &user, DEFAULT_BUCKET, &obj).await
    }

    match bucket_on {
        true => {
            info!("open gateway route: /{{user}}/{{bucket}}/{{object_path}}");
            Router::new()
                .route("/{user}/{bucket}/{*object_path}", get(h_user_bucket))
                .with_state(state)
        }
        false => {
            info!("open gateway route: /{{user}}/{{object_path}}");
            Router::new()
                .route("/{user}/{*object_path}", get(h_user))
                .with_state(state)
        }
    }
}

async fn serve_open_gw(state: &S3Server, user: &str, bucket: &str, object_path: &str) -> Response {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let allowed = state.policy.read().await.allowed(user, bucket, object_path);
    if !allowed {
        warn!(%user, %bucket, %object_path, "open gateway denied");
        return (StatusCode::FORBIDDEN, "access denied").into_response();
    }
    debug!(%user, %bucket, %object_path, "open gateway allowed");
    match state.mounts.open(bucket, object_path) {
        Ok((mut f, _obj)) => {
            let mut buf = Vec::new();
            if std::io::Read::read_to_end(&mut f, &mut buf).is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            (StatusCode::OK, buf).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
