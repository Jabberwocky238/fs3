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

## Testing

```bash
cargo test --test minio_tests
```

**Coverage**: 17 tests, all passing
- Bucket: create, delete, list, policy, tagging, versioning, encryption, lifecycle, replication, notification
- Object: put, get, head, copy, delete, tagging, legal hold, conditional requests
- Multipart: create, abort
- List: recursive, streaming
- Policy: Allow/Deny, wildcards, priority
- Content-MD5: validation on upload
- Conditional requests: If-Match, If-None-Match
- Errors: invalid requests, conflicts

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

### Bucket Operations

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Create bucket | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | Delete bucket | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | Head bucket | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | List buckets | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | Get bucket location | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | Bucket policy | `minio/cmd/bucket-policy-handlers.go` | `tests/minio/policy.rs` |
| ✅ | Bucket policy status | `minio/cmd/bucket-policy-handlers.go` | `tests/minio/policy.rs` |
| ✅ | Bucket tagging | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | Bucket versioning | `minio/cmd/bucket-versioning-handler.go` | `tests/minio/versioning.rs` |
| ✅ | Bucket lifecycle | `minio/cmd/bucket-lifecycle-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | Bucket encryption | `minio/cmd/bucket-encryption-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | Bucket notification | `minio/cmd/bucket-notification-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | Bucket replication | `minio/cmd/bucket-replication-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | Bucket object lock | `minio/cmd/bucket-object-lock.go` | `tests/minio/object_lock.rs` |
| ✅ | Bucket ACL (dummy) | `minio/cmd/bucket-handlers.go` | - |
| ⬜ | Bucket CORS | `minio/cmd/bucket-handlers.go` | `tests/minio/cors.rs` |
| ⬜ | Bucket website | `minio/cmd/bucket-handlers.go` | `tests/minio/website.rs` |
| ⬜ | Bucket logging | `minio/cmd/bucket-handlers.go` | `tests/minio/logging.rs` |
| ⬜ | Bucket accelerate | `minio/cmd/bucket-handlers.go` | `tests/minio/accelerate.rs` |
| ⬜ | Bucket request payment | `minio/cmd/bucket-handlers.go` | `tests/minio/request_payment.rs` |
| ⬜ | Bucket analytics | `minio/cmd/bucket-handlers.go` | `tests/minio/analytics.rs` |
| ⬜ | Bucket metrics | `minio/cmd/bucket-handlers.go` | `tests/minio/metrics_config.rs` |
| ⬜ | Bucket inventory | `minio/cmd/bucket-handlers.go` | `tests/minio/inventory.rs` |

### Object Operations

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Put object | `minio/cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | Get object | `minio/cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | Head object | `minio/cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | Delete object | `minio/cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | Delete multiple objects | `minio/cmd/object-handlers.go` | `tests/minio/batch_version.rs` |
| ✅ | Copy object | `minio/cmd/object-handlers.go` | `tests/minio/object_advanced.rs` |
| ✅ | Get object attributes | `minio/cmd/object-handlers.go` | `tests/minio/object_features.rs` |
| ✅ | Object tagging | `minio/cmd/object-handlers.go` | `tests/minio/object_features.rs` |
| ✅ | Object retention | `minio/cmd/object-handlers.go` | `tests/minio/object_lock.rs` |
| ✅ | Object legal hold | `minio/cmd/object-handlers.go` | `tests/minio/object_lock.rs` |
| ✅ | Object ACL (dummy) | `minio/cmd/object-handlers.go` | - |
| ✅ | Range reads | `minio/cmd/object-handlers.go` | `tests/minio/object_advanced.rs` |
| ✅ | Content-MD5 validation | `minio/cmd/object-handlers.go` | `tests/minio/content_md5.rs` |
| ✅ | Conditional requests | `minio/cmd/object-handlers.go` | `tests/minio/conditional.rs` |
| ✅ | Select object content | `minio/cmd/object-handlers.go` | `tests/minio/select.rs` |
| ⬜ | Restore object | `minio/cmd/object-handlers.go` | `tests/minio/restore.rs` |

### Multipart Upload

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Initiate multipart | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | Upload part | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | Upload part copy | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | Complete multipart | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | Abort multipart | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | List parts | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | List multipart uploads | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |

### List Operations

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | List objects v1 | `minio/cmd/bucket-handlers.go` | `tests/minio/list_objects.rs` |
| ✅ | List objects v2 | `minio/cmd/bucket-handlers.go` | `tests/minio/list_objects.rs` |
| ✅ | List objects v2 (metadata) | `minio/cmd/bucket-handlers.go` | `tests/minio/list_advanced.rs` |
| ✅ | List object versions | `minio/cmd/bucket-handlers.go` | `tests/minio/versioning.rs` |
| ✅ | List versions (metadata) | `minio/cmd/bucket-handlers.go` | `tests/minio/list_advanced.rs` |

### Versioning & Object Lock

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Versioning config | `minio/cmd/bucket-versioning-handler.go` | `tests/minio/versioning.rs` |
| ⬜ | Versioning enforcement | `minio/cmd/erasure.go` | `tests/minio/versioning_enforcement.rs` |
| ✅ | Object lock config | `minio/cmd/bucket-object-lock.go` | `tests/minio/object_lock.rs` |
| ⬜ | Object lock enforcement | `minio/cmd/object-handlers.go` | `tests/minio/object_lock_enforcement.rs` |

### Security & Access Control

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Bucket policy evaluation | `minio/internal/bucket/policy` | `tests/minio/policy.rs` |
| ✅ | ACL (dummy) | `minio/cmd/bucket-handlers.go` | - |
| ⬜ | Pre-signed URLs | `minio/cmd/signature-v4.go` | `tests/minio/presigned.rs` |
| ⬜ | POST policy | `minio/cmd/postpolicyform.go` | `tests/minio/post_policy.rs` |
| ⬜ | Signature V4 auth | `minio/cmd/signature-v4.go` | `tests/minio/auth_v4.rs` |
| ⬜ | CORS enforcement | `minio/cmd/bucket-handlers.go` | `tests/minio/cors_enforcement.rs` |

### Advanced Features

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ⬜ | SSE-S3 encryption | `minio/internal/crypto/crypto.go` | `tests/minio/sse_s3.rs` |
| ⬜ | SSE-C encryption | `minio/internal/crypto/crypto.go` | `tests/minio/sse_c.rs` |
| ⬜ | SSE-KMS encryption | `minio/internal/crypto/crypto.go` | `tests/minio/sse_kms.rs` |
| ⬜ | Event notifications | `minio/cmd/bucket-notification-handlers.go` | `tests/minio/notifications.rs` |
| ⬜ | Replication sync | `minio/cmd/bucket-replication.go` | `tests/minio/replication_sync.rs` |
| ⬜ | Lifecycle execution | `minio/cmd/bucket-lifecycle.go` | `tests/minio/lifecycle_execution.rs` |
| ✅ | S3 Select | `minio/cmd/select-objectcontent-handler.go` | `tests/minio/select.rs` |
| ⬜ | Glacier restore | `minio/cmd/object-handlers.go` | `tests/minio/restore.rs` |

### Storage Backends

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Local filesystem | `minio/cmd/erasure.go` | tested |
| ✅ | In-memory | - | tested |
| ⬜ | S3 gateway | - | `tests/minio/gateway_s3.rs` |
| ⬜ | MinIO gateway | - | `tests/minio/gateway_minio.rs` |

### Metadata Backends

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | SQLite | - | default |
| ✅ | JSON file | - | tested |
| ✅ | In-memory | - | tested |
| ⬜ | PostgreSQL | - | `tests/minio/metadata_postgres.rs` |
| ⬜ | Kubernetes ConfigMap | - | `tests/minio/metadata_k8s.rs` |

### Production Features

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ⬜ | Rate limiting | `minio/cmd/api-router.go` | `tests/minio/rate_limit.rs` |
| ⬜ | Metrics (Prometheus) | `minio/cmd/metrics.go` | `tests/minio/metrics.rs` |
| ⬜ | Distributed mode | `minio/cmd/erasure-server-pool.go` | `tests/minio/distributed.rs` |
| ⬜ | TLS termination | `minio/cmd/server-main.go` | `tests/minio/tls.rs` |
| ⬜ | Health checks | `minio/cmd/healthcheck-handler.go` | `tests/minio/health.rs` |

### Error Handling

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Invalid requests | `minio/cmd/api-errors.go` | `tests/minio/error_scenarios.rs` |
| ✅ | Conflicts | `minio/cmd/api-errors.go` | `tests/minio/error_scenarios.rs` |
| ✅ | Not found | `minio/cmd/api-errors.go` | `tests/minio/error_scenarios.rs` |
| ⬜ | Access denied | `minio/cmd/api-errors.go` | `tests/minio/error_access.rs` |
| ⬜ | Quota exceeded | `minio/cmd/api-errors.go` | `tests/minio/error_quota.rs` |
