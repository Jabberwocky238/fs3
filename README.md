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

**Progress**: ⬜ Not Started | 🚧 In Progress | 🧪 Testing | ✅ Done | 🐛 Bug

| Status | Progress | From | Group | Feature | MinIO Source | Test File |
|--------|----------|------|-------|---------|--------------|-----------|
| ✅ | ✅ | AWS S3 | Bucket | Create bucket | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Delete bucket | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Head bucket | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | ✅ | AWS S3 | Bucket | List buckets | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Get bucket location | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Bucket policy | `minio/cmd/bucket-policy-handlers.go` | `tests/minio/policy.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Bucket policy status | `minio/cmd/bucket-policy-handlers.go` | `tests/minio/policy.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Bucket tagging | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Bucket versioning | `minio/cmd/bucket-versioning-handler.go` | `tests/minio/versioning.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Bucket lifecycle | `minio/cmd/bucket-lifecycle-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Bucket encryption | `minio/cmd/bucket-encryption-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Bucket notification | `minio/cmd/bucket-notification-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Bucket replication | `minio/cmd/bucket-replication-handlers.go` | `tests/minio/bucket_config.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Bucket object lock | `minio/cmd/bucket-object-lock.go` | `tests/minio/object_lock.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Bucket ACL (dummy) | `minio/cmd/bucket-handlers.go` | - |
| ⬜ | ⬜ | AWS S3 | Bucket | Bucket CORS | `minio/cmd/bucket-handlers.go` | `tests/minio/cors.rs` |
| ⬜ | ⬜ | AWS S3 | Bucket | Bucket website | `minio/cmd/bucket-handlers.go` | `tests/minio/website.rs` |
| ⬜ | ⬜ | AWS S3 | Bucket | Bucket logging | `minio/cmd/bucket-handlers.go` | `tests/minio/logging.rs` |
| ⬜ | ⬜ | AWS S3 | Bucket | Bucket accelerate | `minio/cmd/bucket-handlers.go` | `tests/minio/accelerate.rs` |
| ⬜ | ⬜ | AWS S3 | Bucket | Bucket request payment | `minio/cmd/bucket-handlers.go` | `tests/minio/request_payment.rs` |
| ⬜ | ⬜ | AWS S3 | Bucket | Bucket analytics | `minio/cmd/bucket-handlers.go` | `tests/minio/analytics.rs` |
| ⬜ | ⬜ | AWS S3 | Bucket | Bucket metrics | `minio/cmd/bucket-handlers.go` | `tests/minio/metrics_config.rs` |
| ⬜ | ⬜ | AWS S3 | Bucket | Bucket inventory | `minio/cmd/bucket-handlers.go` | `tests/minio/inventory.rs` |
| ✅ | ✅ | AWS S3 | Object | Put object | `minio/cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | ✅ | AWS S3 | Object | Get object | `minio/cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | ✅ | AWS S3 | Object | Head object | `minio/cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | ✅ | AWS S3 | Object | Delete object | `minio/cmd/object-handlers.go` | `tests/minio/object.rs` |
| ✅ | ✅ | AWS S3 | Object | Delete multiple objects | `minio/cmd/object-handlers.go` | `tests/minio/batch_version.rs` |
| ✅ | ✅ | AWS S3 | Object | Copy object | `minio/cmd/object-handlers.go` | `tests/minio/object_advanced.rs` |
| ✅ | ✅ | AWS S3 | Object | Get object attributes | `minio/cmd/object-handlers.go` | `tests/minio/object_features.rs` |
| ✅ | ✅ | AWS S3 | Object | Object tagging | `minio/cmd/object-handlers.go` | `tests/minio/object_features.rs` |
| ✅ | ✅ | AWS S3 | Object | Object retention | `minio/cmd/object-handlers.go` | `tests/minio/object_lock.rs` |
| ✅ | ✅ | AWS S3 | Object | Object legal hold | `minio/cmd/object-handlers.go` | `tests/minio/object_lock.rs` |
| ✅ | ✅ | AWS S3 | Object | Object ACL (dummy) | `minio/cmd/object-handlers.go` | - |
| ✅ | ✅ | AWS S3 | Object | Range reads | `minio/cmd/object-handlers.go` | `tests/minio/object_advanced.rs` |
| ✅ | ✅ | AWS S3 | Object | Content-MD5 validation | `minio/cmd/object-handlers.go` | `tests/minio/content_md5.rs` |
| ✅ | ✅ | AWS S3 | Object | Conditional requests | `minio/cmd/object-handlers.go` | `tests/minio/conditional.rs` |
| ✅ | ✅ | AWS S3 | Object | Select object content | `minio/cmd/object-handlers.go` | `tests/minio/select.rs` |
| ⬜ | ⬜ | AWS S3 | Object | Restore object | `minio/cmd/object-handlers.go` | `tests/minio/restore.rs` |
| ✅ | ✅ | AWS S3 | Multipart | Initiate multipart | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | ✅ | AWS S3 | Multipart | Upload part | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | ✅ | AWS S3 | Multipart | Upload part copy | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | ✅ | AWS S3 | Multipart | Complete multipart | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | ✅ | AWS S3 | Multipart | Abort multipart | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | ✅ | AWS S3 | Multipart | List parts | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | ✅ | AWS S3 | Multipart | List multipart uploads | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | ✅ | AWS S3 | List | List objects v1 | `minio/cmd/bucket-handlers.go` | `tests/minio/list_objects.rs` |
| ✅ | ✅ | AWS S3 | List | List objects v2 | `minio/cmd/bucket-handlers.go` | `tests/minio/list_objects.rs` |
| ✅ | ✅ | AWS S3 | List | List objects v2 (metadata) | `minio/cmd/bucket-handlers.go` | `tests/minio/list_advanced.rs` |
| ✅ | ✅ | AWS S3 | List | List object versions | `minio/cmd/bucket-handlers.go` | `tests/minio/versioning.rs` |
| ✅ | ✅ | AWS S3 | List | List versions (metadata) | `minio/cmd/bucket-handlers.go` | `tests/minio/list_advanced.rs` |
| ✅ | ✅ | AWS S3 | Versioning | Versioning config | `minio/cmd/bucket-versioning-handler.go` | `tests/minio/versioning.rs` |
| ⬜ | ⬜ | AWS S3 | Versioning | Versioning enforcement | `minio/cmd/erasure.go` | `tests/minio/versioning_enforcement.rs` |
| ✅ | ✅ | AWS S3 | Versioning | Object lock config | `minio/cmd/bucket-object-lock.go` | `tests/minio/object_lock.rs` |
| ⬜ | ⬜ | AWS S3 | Versioning | Object lock enforcement | `minio/cmd/object-handlers.go` | `tests/minio/object_lock_enforcement.rs` |
| ✅ | ✅ | AWS S3 | Security | Bucket policy evaluation | `minio/internal/bucket/policy` | `tests/minio/policy.rs` |
| ✅ | ✅ | AWS S3 | Security | ACL (dummy) | `minio/cmd/bucket-handlers.go` | - |
| ⬜ | ⬜ | AWS S3 | Security | Pre-signed URLs | `minio/cmd/signature-v4.go` | `tests/minio/presigned.rs` |
| ⬜ | ⬜ | AWS S3 | Security | POST policy | `minio/cmd/postpolicyform.go` | `tests/minio/post_policy.rs` |
| ⬜ | ⬜ | AWS S3 | Security | Signature V4 auth | `minio/cmd/signature-v4.go` | `tests/minio/auth_v4.rs` |
| ⬜ | ⬜ | AWS S3 | Security | CORS enforcement | `minio/cmd/bucket-handlers.go` | `tests/minio/cors_enforcement.rs` |
| ⬜ | ⬜ | AWS S3 | Advanced | SSE-S3 encryption | `minio/internal/crypto/crypto.go` | `tests/minio/sse_s3.rs` |
| ⬜ | ⬜ | AWS S3 | Advanced | SSE-C encryption | `minio/internal/crypto/crypto.go` | `tests/minio/sse_c.rs` |
| ⬜ | ⬜ | AWS S3 | Advanced | SSE-KMS encryption | `minio/internal/crypto/crypto.go` | `tests/minio/sse_kms.rs` |
| ⬜ | ⬜ | AWS S3 | Advanced | Event notifications | `minio/cmd/bucket-notification-handlers.go` | `tests/minio/notifications.rs` |
| ⬜ | ⬜ | AWS S3 | Advanced | Replication sync | `minio/cmd/bucket-replication.go` | `tests/minio/replication_sync.rs` |
| ⬜ | ⬜ | AWS S3 | Advanced | Lifecycle execution | `minio/cmd/bucket-lifecycle.go` | `tests/minio/lifecycle_execution.rs` |
| ✅ | ✅ | AWS S3 | Advanced | S3 Select | `minio/cmd/select-objectcontent-handler.go` | `tests/minio/select.rs` |
| ⬜ | ⬜ | AWS S3 | Advanced | Glacier restore | `minio/cmd/object-handlers.go` | `tests/minio/restore.rs` |
| ⬜ | ⬜ | MinIO | Storage | Erasure coding (EC) | `minio/cmd/erasure-coding.go` | `tests/minio/erasure_coding.rs` |
| ⬜ | ⬜ | MinIO | Storage | Multi-disk sets | `minio/cmd/erasure-sets.go` | `tests/minio/erasure_sets.rs` |
| ⬜ | ⬜ | MinIO | Storage | Server pools | `minio/cmd/erasure-server-pool.go` | `tests/minio/server_pools.rs` |
| ⬜ | ⬜ | MinIO | Storage | Healing & repair | `minio/cmd/erasure-healing.go` | `tests/minio/healing.rs` |
| ⬜ | ⬜ | MinIO | Storage | Bitrot protection | `minio/cmd/erasure.go` | `tests/minio/bitrot.rs` |
| ⬜ | ⬜ | MinIO | Metadata | xl.meta per object | `minio/cmd/xl-storage.go` | `tests/minio/xl_meta.rs` |
| ⬜ | ⬜ | MinIO | Metadata | .metadata.bin per bucket | `minio/cmd/bucket-metadata.go` | `tests/minio/bucket_metadata.rs` |
| ⬜ | ⬜ | MinIO | Production | Rate limiting | `minio/cmd/api-router.go` | `tests/minio/rate_limit.rs` |
| ⬜ | ⬜ | MinIO | Production | Metrics (Prometheus) | `minio/cmd/metrics.go` | `tests/minio/metrics.rs` |
| ⬜ | ⬜ | MinIO | Production | Distributed mode | `minio/cmd/erasure-server-pool.go` | `tests/minio/distributed.rs` |
| ⬜ | ⬜ | MinIO | Production | TLS termination | `minio/cmd/server-main.go` | `tests/minio/tls.rs` |
| ⬜ | ⬜ | MinIO | Production | Health checks | `minio/cmd/healthcheck-handler.go` | `tests/minio/health.rs` |
| ✅ | ✅ | AWS S3 | Error | Invalid requests | `minio/cmd/api-errors.go` | `tests/minio/error_scenarios.rs` |
| ✅ | ✅ | AWS S3 | Error | Conflicts | `minio/cmd/api-errors.go` | `tests/minio/error_scenarios.rs` |
| ✅ | ✅ | AWS S3 | Error | Not found | `minio/cmd/api-errors.go` | `tests/minio/error_scenarios.rs` |
| ⬜ | ⬜ | AWS S3 | Error | Access denied | `minio/cmd/api-errors.go` | `tests/minio/error_access.rs` |
| ⬜ | ⬜ | AWS S3 | Error | Quota exceeded | `minio/cmd/api-errors.go` | `tests/minio/error_quota.rs` |
