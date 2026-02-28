use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::response::Response;
use axum::Router;
use chrono::{DateTime, Utc};
use clap::Parser;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

use crate::config::{Config, MountMode};
use crate::mount::{MountManager, new_mount};
use crate::policy::PolicyEngine;
use crate::storage::factory::{self, StorageBackend};
#[cfg(feature = "policy")]
use crate::storage::PolicyStore;
use crate::storage::types::StorageSnapshot;
use crate::storage::types::UserRecord;

// ── S3Server ────────────────────────────────────────────────────────────

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
    pub listen_outer: String,
    pub presigned: Arc<Mutex<HashMap<String, SignedToken>>>,
    pub uploads: Arc<Mutex<HashMap<String, MultipartUpload>>>,
}

#[derive(Parser)]
#[command(name = "fs3", about = "S3-compatible mount gateway")]
struct Cli {
    /// Config file path (JSON)
    #[arg(short, long)]
    config: Option<String>,

    /// Inner listen address
    #[arg(short, long)]
    listen: Option<String>,

    /// Outer listen address
    #[arg(long)]
    listen_outer: Option<String>,

    /// Mount mode
    #[arg(short, long, value_enum)]
    mount: Option<MountMode>,

    /// Mount path
    #[arg(long)]
    mount_path: Option<String>,
}

pub async fn run_from_cli() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    let mut cfg = match cli.config {
        Some(ref path) => {
            info!(path = %path, "loading config file");
            load_config(path)?
        }
        None => {
            info!("no config file specified, using defaults");
            Config::default()
        }
    };

    if let Some(v) = cli.listen { cfg.listen_inner = v; }
    if let Some(v) = cli.listen_outer { cfg.listen_outer = v; }
    if let Some(v) = cli.mount { cfg.mount.mode = v; }
    if let Some(v) = cli.mount_path { cfg.mount.path = v; }

    info!(listen_inner = %cfg.listen_inner, listen_outer = %cfg.listen_outer,
          mount_mode = ?cfg.mount.mode, storage_kind = ?cfg.storage.kind,
          user_enabled = cfg.user_enabled, "resolved config");

    S3Server::from_config(cfg).await?.serve().await
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
        let mounts = Arc::from(
            new_mount(&cfg.mount).context("init mounts")?,
        );

        let seed = StorageSnapshot {
            users: vec![UserRecord {
                user_id: "alice".into(),
                enabled: true,
                access_key: "alice-ak".into(),
                secret_key: "alice-sk".into(),
                groups: vec!["groupA".into()],
                attrs: std::collections::HashMap::new(),
            }],
            policy_groups: vec![],
            bucket_metadata: vec![],
        };
        info!(kind = ?cfg.storage.kind, "initializing storage backend");
        let store = factory::new_store(&cfg.storage, seed)
            .await
            .context("init metadata store")?;
        #[cfg(feature = "policy")]
        let groups = store
            .list_policy_groups()
            .await
            .context("load policy groups")?;
        #[cfg(not(feature = "policy"))]
        let groups = vec![];

        let listen_outer = normalize_outer(&cfg.listen_outer);
        Ok(S3Server {
            mounts,
            policy: Arc::new(RwLock::new(PolicyEngine::new(&groups))),
            store,
            cfg,
            listen_outer,
            presigned: Arc::new(Mutex::new(Default::default())),
            uploads: Arc::new(Mutex::new(Default::default())),
        })
    }

    pub async fn serve(self) -> Result<()> {
        let ctrl_router: Router = ctrl_gw_router(self.clone())
            .merge(crate::s3::router(self.clone()));
        let open_router: Router = open_gw_router(self.clone());

        let ctrl_addr = normalize_listen(&self.cfg.listen_inner);
        let open_addr = normalize_listen(&self.cfg.listen_outer);

        let ctrl_listener = TcpListener::bind(&ctrl_addr).await?;
        let open_listener = TcpListener::bind(&open_addr).await?;

        info!("control gateway listening on {}", ctrl_addr);
        info!("open gateway listening on {}", open_addr);

        let ctrl_task =
            tokio::spawn(async move { axum::serve(ctrl_listener, ctrl_router).await });
        let open_task =
            tokio::spawn(async move { axum::serve(open_listener, open_router).await });

        tokio::select! {
            res = ctrl_task => { res??; }
            res = open_task => { res??; }
        }
        Ok(())
    }
}

// ── helpers ────────────────────────────────────────────────────────────

fn load_config(path: &str) -> Result<Config> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("reading config file: {}", path))?;
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
        addr.trim_end_matches('/').to_string()
    }
}

fn ctrl_gw_router(state: S3Server) -> Router {
    use axum::routing::get;
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .with_state(state)
}

fn open_gw_router(state: S3Server) -> Router {
    use axum::extract::{Path, State};
    use axum::response::IntoResponse;
    use axum::routing::get;

    async fn open_gw_with_user(
        State(state): State<S3Server>,
        Path((user, bucket, object_path)): Path<(String, String, String)>,
    ) -> impl IntoResponse {
        serve_open_gw(&state, &user, &bucket, &object_path).await
    }

    async fn open_gw_no_user(
        State(state): State<S3Server>,
        Path((bucket, object_path)): Path<(String, String)>,
    ) -> impl IntoResponse {
        serve_open_gw(&state, "", &bucket, &object_path).await
    }

    if state.cfg.user_enabled {
        info!("open gateway route: /{{user}}/{{bucket}}/{{object_path}}");
        Router::new()
            .route("/{user}/{bucket}/{*object_path}", get(open_gw_with_user))
            .with_state(state)
    } else {
        info!("open gateway route: /{{bucket}}/{{object_path}}");
        Router::new()
            .route("/{bucket}/{*object_path}", get(open_gw_no_user))
            .with_state(state)
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
