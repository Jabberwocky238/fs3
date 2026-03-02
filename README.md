# FS3

A lightweight, cloud-native object storage gateway written in Rust. Fully compatible with Amazon S3 and MinIO APIs.

## Features

- **S3-Compatible API** — Works with any S3 client (AWS SDK, MinIO SDK, s3cmd, etc.)
- **Pluggable Storage Backends** — Local filesystem, in-memory, or gateway to existing S3/MinIO
- **Pluggable Metadata Storage** — SQLite (default), JSON file, in-memory, or PostgreSQL
- **Multipart Upload** — Full support including proper S3-compatible ETag computation
- **Bucket Operations** — Create, delete, list, versioning, tagging, lifecycle, replication
- **Object Operations** — Put, get, head, copy, delete, batch delete, tagging, retention, legal hold
- **Cloud-Native** — Built with Axum and Tokio for high-performance async I/O

## Quick Start

```bash
# Build
cargo build --release

# Run with default settings (SQLite metadata + local filesystem)
./target/release/s3_mount_gateway_rust

# Run with PostgreSQL metadata storage
cargo build --release --features storage-postgres
```

## Architecture

```
┌─────────────────────────────────┐
│        S3 / MinIO Clients       │
└──────────────┬──────────────────┘
               │ HTTP (S3 API)
┌──────────────▼──────────────────┐
│       Axum HTTP Handler         │
│    (s3_axum_handler)            │
├─────────────────────────────────┤
│         S3 Engine               │
│    (s3_engine)                  │
├──────────┬──────────────────────┤
│ Metadata │       Mount          │
│ Storage  │   (s3_mount)         │
│ (sqlite/ │  (local/memory/      │
│  json/   │   gateway)           │
│  memory) │                      │
└──────────┴──────────────────────┘
```

## Configuration

FS3 uses CLI arguments via `clap`. Run with `--help` for available options.

## Optional Features

| Feature              | Description                    |
|----------------------|--------------------------------|
| `storage-postgres`   | PostgreSQL metadata backend    |
| `storage-k8sconfigmap` | Kubernetes ConfigMap backend |

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Roadmap

### Milestone 1 — Core S3 API (Done)

- [x] Bucket CRUD (create, delete, head, list)
- [x] Object CRUD (put, get, head, copy, delete, batch delete)
- [x] Multipart upload (initiate, upload part, copy part, complete, abort, list)
- [x] S3-compatible ETag (MD5 for single-part, `MD5(parts)-N` for multipart)
- [x] Object tagging, retention, legal hold
- [x] Bucket policy, tagging, versioning config, lifecycle config, encryption config
- [x] Notification, replication, object-lock config storage
- [x] Range reads
- [x] SQLite and JSON metadata backends
- [x] Local filesystem and in-memory mount backends

### Milestone 2 — Enforcement & Correctness

- [ ] Object versioning enforcement (currently config-only)
- [ ] ACL enforcement (currently returns defaults)
- [ ] CORS enforcement on responses
- [ ] Object lock enforcement (WORM)
- [ ] Lifecycle rule execution (expiration, transition)
- [ ] Pre-signed URL support
- [ ] Content-MD5 validation on upload
- [ ] Conditional requests (If-Match, If-None-Match)

### Milestone 3 — Advanced Features

- [ ] Server-side encryption (SSE-S3, SSE-C)
- [ ] Event notifications (SNS/SQS/Kafka)
- [ ] S3 Select (SelectObjectContent)
- [ ] POST policy (form-based upload)
- [ ] Bucket replication (active sync)
- [ ] Glacier-style restore

### Milestone 4 — Production Readiness

- [ ] PostgreSQL metadata backend
- [ ] Kubernetes ConfigMap metadata backend
- [ ] Authentication (Signature V4)
- [ ] Rate limiting and request throttling
- [ ] Metrics and observability (Prometheus)
- [ ] Distributed mode (multi-node)
- [ ] TLS termination
- [ ] Docker image and Helm chart
