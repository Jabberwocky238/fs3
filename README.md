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

## Configuration

FS3 uses CLI arguments via `clap`. Run with `--help` for available options.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Roadmap

**Status**: ⬜ Under Dev | ✅ READY TO USE
**Progress**: ⬜ Not Started | 🚧 In Progress | 🧪 Testing | ✅ Done | 🐛 Bug

| Status | Progress | From | Group | Feature | MinIO Source | Test File |
|--------|----------|------|-------|---------|--------------|-----------|
| ✅ | ✅ | AWS S3 | Bucket | Basic ops (create/delete/head/list/location) | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Policy & policy status | `minio/cmd/bucket-policy-handlers.go` | `tests/minio/policy.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Config (tagging/lifecycle/encryption/notification/replication) | Various handlers | `tests/minio/bucket_config.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Versioning | `minio/cmd/bucket-versioning-handler.go` | `tests/minio/versioning.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Object lock | `minio/cmd/bucket-object-lock.go` | `tests/minio/object_lock.rs` |
| ✅ | ✅ | AWS S3 | Bucket | Website | `minio/cmd/bucket-handlers.go` | `tests/aws/website.rs` |
| ⬜ | ⬜ | AWS S3 | Bucket | CORS/logging/accelerate/payment/analytics/metrics/inventory | `minio/cmd/bucket-handlers.go` | Various |
| 🚧 | 🚧 | AWS S3 | Object | Basic ops (put/get/head/delete/copy) | `minio/cmd/object-handlers.go` | `tests/minio/object.rs` |
| 🚧 | 🚧 | AWS S3 | Object | Advanced (attributes/tagging/retention/legal hold/range/MD5/conditional/select) | `minio/cmd/object-handlers.go` | Various |
| ✅ | ✅ | AWS S3 | Multipart | Full support (initiate/upload/complete/abort) | `minio/cmd/object-multipart-handlers.go` | `tests/aws/multipart.rs` |
| 🚧 | 🚧 | AWS S3 | Multipart | Upload part copy, list parts/uploads | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ✅ | ✅ | AWS S3 | List | List objects v1/v2, versions (with metadata) | `minio/cmd/bucket-handlers.go` | Various |
| ⬜ | ⬜ | AWS S3 | Versioning | Enforcement & object lock enforcement | Various | Various |
| ✅ | ✅ | AWS S3 | Security | Policy evaluation, ACL (dummy) | `minio/internal/bucket/policy` | `tests/minio/policy.rs` |
| ⬜ | ⬜ | AWS S3 | Security | Pre-signed URLs, POST policy, Signature V4, CORS | Various | Various |
| ⬜ | ⬜ | AWS S3 | Advanced | SSE encryption (S3/C/KMS) | `minio/internal/crypto/crypto.go` | Various |
| ⬜ | ⬜ | AWS S3 | Advanced | Event notifications, replication, lifecycle execution | Various | Various |
| ✅ | ✅ | AWS S3 | Advanced | S3 Select | `minio/cmd/select-objectcontent-handler.go` | `tests/minio/select.rs` |
| ⬜ | ⬜ | MinIO | Storage | Erasure coding, multi-disk, server pools, healing, bitrot | Various | Various |
| ⬜ | ⬜ | MinIO | Metadata | xl.meta, .metadata.bin | Various | Various |
| ⬜ | ⬜ | MinIO | Production | Rate limiting, metrics, distributed mode | Various | Various |

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
