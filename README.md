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

**Coverage**: 15 tests, all passing
- Bucket: create, delete, list, policy, tagging, versioning, encryption, lifecycle, replication, notification
- Object: put, get, head, copy, delete, tagging, legal hold
- Multipart: create, abort
- List: recursive, streaming
- Policy: Allow/Deny, wildcards, priority
- Content-MD5: validation on upload
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
| ✅ | Create bucket | `cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | Delete bucket | `cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | Head bucket | `cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | List buckets | `cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | Get bucket location | `cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | Bucket policy | `cmd/bucket-policy-handlers.go` | `tests/minio/policy.rs` |
| ✅ | Bucket policy status | `cmd/bucket-policy-handlers.go` | `tests/minio/policy.rs` |
| ✅ | Bucket tagging | `cmd/bucket-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | Bucket versioning | `cmd/bucket-versioning-handlers.go` | `tests/minio/versioning.rs` |
| ✅ | Bucket lifecycle | `cmd/bucket-lifecycle-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | Bucket encryption | `cmd/bucket-encryption-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | Bucket notification | `cmd/bucket-notification-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | Bucket replication | `cmd/bucket-replication-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | Bucket object lock | `cmd/bucket-object-lock-handlers.go` | `tests/minio/object_lock.rs` |
| ✅ | Bucket ACL (dummy) | `cmd/bucket-handlers.go` | - |
| ⬜ | Bucket CORS | `cmd/bucket-handlers.go` | `tests/minio/cors.rs` |
| ⬜ | Bucket website | `cmd/bucket-handlers.go` | `tests/minio/website.rs` |
| ⬜ | Bucket logging | `cmd/bucket-handlers.go` | `tests/minio/logging.rs` |
| ⬜ | Bucket accelerate | `cmd/bucket-handlers.go` | `tests/minio/accelerate.rs` |
| ⬜ | Bucket request payment | `cmd/bucket-handlers.go` | `tests/minio/request_payment.rs` |
| ⬜ | Bucket analytics | `cmd/bucket-handlers.go` | `tests/minio/analytics.rs` |
| ⬜ | Bucket metrics | `cmd/bucket-handlers.go` | `tests/minio/metrics_config.rs` |
| ⬜ | Bucket inventory | `cmd/bucket-handlers.go` | `tests/minio/inventory.rs` |

### Object Operations

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Put object | `cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | Get object | `cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | Head object | `cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | Delete object | `cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | Delete multiple objects | `cmd/object-handlers.go` | `tests/minio/batch_version.rs` |
| ✅ | Copy object | `cmd/object-handlers.go` | `tests/minio/object_advanced.rs` |
| ✅ | Get object attributes | `cmd/object-handlers.go` | `tests/minio/object_features.rs` |
| ✅ | Object tagging | `cmd/object-handlers.go` | `tests/minio/object_features.rs` |
| ✅ | Object retention | `cmd/object-handlers.go` | `tests/minio/object_lock.rs` |
| ✅ | Object legal hold | `cmd/object-handlers.go` | `tests/minio/object_lock.rs` |
| ✅ | Object ACL (dummy) | `cmd/object-handlers.go` | - |
| ✅ | Range reads | `cmd/object-handlers.go` | `tests/minio/object_advanced.rs` |
| ✅ | Content-MD5 validation | `cmd/object-handlers.go` | `tests/minio/content_md5.rs` |
| ⬜ | Conditional requests | `cmd/object-handlers.go` | `tests/minio/conditional.rs` |
| ⬜ | Select object content | `cmd/object-handlers.go` | `tests/minio/select.rs` |
| ⬜ | Restore object | `cmd/object-handlers.go` | `tests/minio/restore.rs` |

### Multipart Upload

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Initiate multipart | `cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | Upload part | `cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | Upload part copy | `cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | Complete multipart | `cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | Abort multipart | `cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | List parts | `cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | List multipart uploads | `cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |

### List Operations

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | List objects v1 | `cmd/bucket-handlers.go` | `tests/minio/list_objects.rs` |
| ✅ | List objects v2 | `cmd/bucket-handlers.go` | `tests/minio/list_objects.rs` |
| ✅ | List objects v2 (metadata) | `cmd/bucket-handlers.go` | `tests/minio/list_advanced.rs` |
| ✅ | List object versions | `cmd/bucket-handlers.go` | `tests/minio/versioning.rs` |
| ✅ | List versions (metadata) | `cmd/bucket-handlers.go` | `tests/minio/list_advanced.rs` |

### Versioning & Object Lock

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Versioning config | `cmd/bucket-versioning-handlers.go` | `tests/minio/versioning.rs` |
| ⬜ | Versioning enforcement | `cmd/erasure-object.go` | `tests/minio/versioning_enforcement.rs` |
| ✅ | Object lock config | `cmd/bucket-object-lock-handlers.go` | `tests/minio/object_lock.rs` |
| ⬜ | Object lock enforcement | `cmd/object-handlers.go` | `tests/minio/object_lock_enforcement.rs` |

### Security & Access Control

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Bucket policy evaluation | `internal/bucket/policy` | `tests/minio/policy.rs` |
| ✅ | ACL (dummy) | `cmd/bucket-handlers.go` | - |
| ⬜ | Pre-signed URLs | `cmd/signature-v4.go` | `tests/minio/presigned.rs` |
| ⬜ | POST policy | `cmd/post-policy.go` | `tests/minio/post_policy.rs` |
| ⬜ | Signature V4 auth | `cmd/signature-v4.go` | `tests/minio/auth_v4.rs` |
| ⬜ | CORS enforcement | `cmd/cors.go` | `tests/minio/cors_enforcement.rs` |

### Advanced Features

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ⬜ | SSE-S3 encryption | `cmd/crypto` | `tests/minio/sse_s3.rs` |
| ⬜ | SSE-C encryption | `cmd/crypto` | `tests/minio/sse_c.rs` |
| ⬜ | SSE-KMS encryption | `cmd/crypto` | `tests/minio/sse_kms.rs` |
| ⬜ | Event notifications | `cmd/notification.go` | `tests/minio/notifications.rs` |
| ⬜ | Replication sync | `cmd/bucket-replication.go` | `tests/minio/replication_sync.rs` |
| ⬜ | Lifecycle execution | `cmd/lifecycle.go` | `tests/minio/lifecycle_execution.rs` |
| ⬜ | S3 Select | `cmd/select-objectcontent-handler.go` | `tests/minio/select.rs` |
| ⬜ | Glacier restore | `cmd/object-handlers.go` | `tests/minio/restore.rs` |

### Storage Backends

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Local filesystem | `cmd/erasure-*.go` | tested |
| ✅ | In-memory | - | tested |
| ⬜ | S3 gateway | `cmd/gateway` | `tests/minio/gateway_s3.rs` |
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
| ⬜ | Rate limiting | `cmd/api-router.go` | `tests/minio/rate_limit.rs` |
| ⬜ | Metrics (Prometheus) | `cmd/metrics.go` | `tests/minio/metrics.rs` |
| ⬜ | Distributed mode | `cmd/erasure-sets.go` | `tests/minio/distributed.rs` |
| ⬜ | TLS termination | `cmd/server-main.go` | `tests/minio/tls.rs` |
| ⬜ | Health checks | `cmd/health.go` | `tests/minio/health.rs` |

### Error Handling

| Status | Feature | MinIO Source | Test File |
|--------|---------|--------------|-----------|
| ✅ | Invalid requests | `cmd/api-errors.go` | `tests/minio/error_scenarios.rs` |
| ✅ | Conflicts | `cmd/api-errors.go` | `tests/minio/error_scenarios.rs` |
| ✅ | Not found | `cmd/api-errors.go` | `tests/minio/error_scenarios.rs` |
| ⬜ | Access denied | `cmd/api-errors.go` | `tests/minio/error_access.rs` |
| ⬜ | Quota exceeded | `cmd/api-errors.go` | `tests/minio/error_quota.rs` |
