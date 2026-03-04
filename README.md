# FS3

A lightweight, cloud-native object storage gateway written in Rust. Fully compatible with Amazon S3 and MinIO APIs.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Roadmap

**Progress**: ⬜ Not Started | 🚧 In Progress | 🧪 Testing | ✅ Done | 🐛 Bug

| Progress | From | Group | Feature | MinIO Source | Test File |
|----------|------|-------|---------|--------------|-----------|
| ⬜ | AWS S3 | Bucket | Basic ops (create/delete/head/list/location) | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket.rs` |
| ⬜ | AWS S3 | Bucket | Policy & policy status | `minio/cmd/bucket-policy-handlers.go` | `tests/minio/policy.rs` |
| ⬜ | AWS S3 | Bucket | Tagging | `minio/cmd/bucket-handlers.go` | `tests/minio/bucket_config.rs` |
| ⬜ | AWS S3 | Bucket | Lifecycle | `minio/cmd/bucket-lifecycle-handlers.go` | `tests/minio/bucket_config.rs` |
| ⬜ | AWS S3 | Bucket | Encryption | `minio/cmd/bucket-encryption-handlers.go` | `tests/minio/bucket_config.rs` |
| ⬜ | AWS S3 | Bucket | Notification | `minio/cmd/bucket-notification-handlers.go` | `tests/minio/bucket_config.rs` |
| ⬜ | AWS S3 | Bucket | Replication | `minio/cmd/bucket-replication-handlers.go` | `tests/minio/bucket_config.rs` |
| ⬜ | AWS S3 | Bucket | Versioning | `minio/cmd/bucket-versioning-handler.go` | `tests/minio/versioning.rs` |
| ⬜ | AWS S3 | Bucket | Object lock | `minio/cmd/bucket-handlers.go` | `tests/minio/object_lock.rs` |
| ⬜ | AWS S3 | Bucket | ACL (get/put) | `minio/cmd/acl-handlers.go` | `tests/minio/acl.rs` |
| ⬜ | AWS S3 | Bucket | CORS (get/put/delete) | `minio/cmd/dummy-handlers.go` | `tests/aws/cors.rs` |
| ⬜ | AWS S3 | Bucket | CORS enforcement | `minio/cmd/dummy-handlers.go` | `tests/aws/cors_enforcement.rs` |
| ⬜ | AWS S3 | Bucket | Website (get/delete) | `minio/cmd/dummy-handlers.go` | `tests/minio/website.rs`, `tests/aws/website.rs` |
| ⬜ | AWS S3 | Bucket | Logging (get) | `minio/cmd/dummy-handlers.go` | `tests/aws/logging.rs` |
| ⬜ | AWS S3 | Bucket | Accelerate (get) | `minio/cmd/dummy-handlers.go` | `tests/aws/accelerate.rs` |
| ⬜ | AWS S3 | Bucket | Request payment (get) | `minio/cmd/dummy-handlers.go` | `tests/aws/request_payment.rs` |
| ⬜ | AWS S3 | Bucket | Analytics | `minio/cmd/dummy-handlers.go` | `tests/aws/analytics.rs` |
| ⬜ | AWS S3 | Bucket | Inventory | `minio/cmd/dummy-handlers.go` | `tests/aws/inventory.rs` |
| ⬜ | AWS S3 | Bucket | Metrics config | `minio/cmd/dummy-handlers.go` | `tests/aws/metrics_config.rs` |
| ⬜ | AWS S3 | Bucket | Basic ops (AWS SDK) | `minio/cmd/bucket-handlers.go` | `tests/aws/bucket.rs` |
| ⬜ | AWS S3 | Bucket | Config (AWS SDK) | `minio/cmd/bucket-handlers.go` | `tests/aws/bucket_config.rs` |
| ⬜ | AWS S3 | Object | Basic ops (put/get/head/delete) | `minio/cmd/object-handlers.go` | `tests/minio/object.rs`, `tests/aws/object.rs` |
| ⬜ | AWS S3 | Object | Copy object | `minio/cmd/object-handlers.go` | `tests/minio/object_advanced.rs`, `tests/aws/object_advanced.rs` |
| ⬜ | AWS S3 | Object | Delete multiple | `minio/cmd/bucket-handlers.go` | `tests/minio/batch_version.rs` |
| ⬜ | AWS S3 | Object | Tagging (get/put/delete) | `minio/cmd/object-handlers.go` | `tests/minio/object_advanced.rs`, `tests/aws/object_advanced.rs` |
| ⬜ | AWS S3 | Object | Retention & legal hold | `minio/cmd/object-handlers.go` | `tests/minio/object_lock.rs`, `tests/aws/object_lock.rs` |
| ⬜ | AWS S3 | Object | ACL (get/put) | `minio/cmd/acl-handlers.go` | `tests/minio/object_acl.rs` |
| ⬜ | AWS S3 | Object | Attributes (get) | `minio/cmd/object-handlers.go` | `tests/minio/object_attributes.rs` |
| ⬜ | AWS S3 | Object | Range reads | `minio/cmd/object-handlers.go` | `tests/minio/object_advanced.rs`, `tests/aws/object_advanced.rs` |
| ⬜ | AWS S3 | Object | Content-MD5 validation | `minio/cmd/object-handlers.go` | `tests/minio/content_md5.rs` |
| ⬜ | AWS S3 | Object | Conditional requests | `minio/cmd/object-handlers.go` | `tests/minio/conditional.rs`, `tests/aws/conditional.rs` |
| ⬜ | AWS S3 | Object | Select content (S3 Select) | `minio/cmd/object-handlers.go` | `tests/minio/select.rs` |
| ⬜ | AWS S3 | Object | Restore (Glacier) | `minio/cmd/object-handlers.go` | `tests/aws/restore.rs` |
| ⬜ | AWS S3 | Object | Features (AWS SDK) | `minio/cmd/object-handlers.go` | `tests/minio/object_features.rs`, `tests/aws/object_features.rs` |
| ⬜ | AWS S3 | Multipart | Core (initiate/upload/complete/abort) | `minio/cmd/object-multipart-handlers.go` | `tests/aws/multipart.rs` |
| ⬜ | AWS S3 | Multipart | Upload part copy | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ⬜ | AWS S3 | Multipart | List parts | `minio/cmd/object-multipart-handlers.go` | `tests/minio/multipart.rs` |
| ⬜ | AWS S3 | Multipart | List uploads | `minio/cmd/bucket-handlers.go` | `tests/minio/multipart.rs` |
| ⬜ | AWS S3 | List | List objects v1/v2 | `minio/cmd/bucket-listobjects-handlers.go` | `tests/minio/list_objects.rs`, `tests/aws/list_objects.rs` |
| ⬜ | AWS S3 | List | List with metadata | `minio/cmd/bucket-listobjects-handlers.go` | `tests/minio/list_advanced.rs` |
| ⬜ | AWS S3 | List | List versions | `minio/cmd/bucket-listobjects-handlers.go` | `tests/minio/versioning.rs` |
| ⬜ | AWS S3 | Versioning | Enforcement | `minio/cmd/erasure-object.go` | `tests/minio/versioning_enforcement.rs`, `tests/aws/versioning_enforcement.rs` |
| ⬜ | AWS S3 | Versioning | Basic (AWS SDK) | `minio/cmd/bucket-versioning-handler.go` | `tests/aws/versioning.rs` |
| ⬜ | AWS S3 | Versioning | Object lock enforcement | `minio/cmd/erasure-object.go` | `tests/minio/object_lock_enforcement.rs` |
| ⬜ | AWS S3 | Security | Policy evaluation | `minio/internal/bucket/policy` | `tests/minio/policy.rs`, `tests/aws/policy.rs` |
| ⬜ | AWS S3 | Security | Policy advanced | `minio/internal/bucket/policy` | `tests/minio/policy_advanced.rs`, `tests/aws/policy_advanced.rs` |
| ⬜ | AWS S3 | Security | Signature V2 auth | `minio/cmd/signature-v2.go` | `tests/minio/auth_v2.rs` |
| ⬜ | AWS S3 | Security | Signature V4 auth | `minio/cmd/signature-v4.go` | `tests/minio/auth_v4.rs` |
| ⬜ | AWS S3 | Security | Pre-signed URLs | `minio/cmd/signature-v4.go` | `tests/minio/presigned.rs`, `tests/aws/presigned.rs` |
| ⬜ | AWS S3 | Security | POST policy | `minio/cmd/postpolicyform.go` | `tests/minio/post_policy.rs` |
| ⬜ | AWS S3 | Security | STS (AssumeRole, etc.) | `minio/cmd/sts-handlers.go` | `tests/minio/sts.rs` |
| ⬜ | AWS S3 | Advanced | SSE-S3 encryption | `minio/cmd/encryption-v1.go` | `tests/minio/sse_s3.rs`, `tests/aws/sse_s3.rs` |
| ⬜ | AWS S3 | Advanced | SSE-C encryption | `minio/cmd/encryption-v1.go` | `tests/minio/sse_c.rs` |
| ⬜ | AWS S3 | Advanced | SSE-KMS encryption | `minio/cmd/encryption-v1.go` | `tests/aws/sse_kms.rs` |
| ⬜ | AWS S3 | Advanced | Event notifications | `minio/cmd/event-notification.go` | `tests/minio/notifications.rs`, `tests/aws/notifications.rs` |
| ⬜ | AWS S3 | Advanced | Replication sync | `minio/cmd/bucket-replication.go` | `tests/minio/replication_sync.rs`, `tests/aws/replication_sync.rs` |
| ⬜ | AWS S3 | Advanced | Lifecycle execution | `minio/cmd/bucket-lifecycle.go` | `tests/minio/lifecycle_execution.rs`, `tests/aws/lifecycle_execution.rs` |
| ⬜ | MinIO | Storage | Erasure coding | `minio/cmd/erasure-coding.go` | `tests/minio/erasure.rs` |
| ⬜ | MinIO | Storage | Bitrot protection | `minio/cmd/erasure-healing.go` | `tests/minio/bitrot.rs` |
| ⬜ | MinIO | Storage | Healing operations | `minio/cmd/erasure-healing.go` | `tests/minio/healing.rs` |
| ⬜ | MinIO | Metadata | xl.meta per object | `minio/cmd/xl-storage.go` | `tests/minio/xl_meta.rs` |
| ⬜ | MinIO | Metadata | Bucket metadata | `minio/cmd/bucket-metadata.go` | `tests/minio/bucket_metadata.rs` |
| ⬜ | MinIO | IAM | User management | `minio/cmd/iam.go` | `tests/minio/iam.rs` |
| ⬜ | MinIO | IAM | Group management | `minio/cmd/iam.go` | `tests/minio/iam_groups.rs` |
| ⬜ | MinIO | IAM | Service accounts | `minio/cmd/iam.go` | `tests/minio/service_accounts.rs` |
| ⬜ | MinIO | Production | Distributed mode (erasure sets) | `minio/cmd/erasure-server-pool.go` | `tests/minio/distributed.rs` |
| ⬜ | MinIO | Production | Server pools | `minio/cmd/erasure-server-pool.go` | `tests/minio/server_pools.rs` |
| ⬜ | MinIO | Production | Decommissioning | `minio/cmd/erasure-server-pool-decom.go` | `tests/minio/decommission.rs` |
| ⬜ | MinIO | Production | Rebalancing | `minio/cmd/erasure-server-pool-rebalance.go` | `tests/minio/rebalance.rs` |
| ⬜ | MinIO | Production | Site replication | `minio/cmd/site-replication.go` | `tests/minio/site_replication.rs` |
| ⬜ | MinIO | Production | TLS termination | `minio/cmd/server-main.go` | `tests/minio/tls.rs` |
| ⬜ | MinIO | Production | Rate limiting | `minio/cmd/api-router.go` | `tests/minio/rate_limit.rs` |
| ⬜ | MinIO | Monitoring | Health checks (liveness/readiness) | `minio/cmd/healthcheck-handler.go` | `tests/minio/health.rs` |
| ⬜ | MinIO | Monitoring | Metrics v3 (Prometheus) | `minio/cmd/metrics-v3.go` | `tests/minio/metrics.rs` |
| ⬜ | MinIO | Monitoring | HTTP tracing | `minio/cmd/http-tracer.go` | `tests/minio/tracing.rs` |
| ⬜ | MinIO | Monitoring | Data scanner | `minio/cmd/data-scanner.go` | `tests/minio/scanner.rs` |
| ⬜ | MinIO | Admin | Bucket admin APIs | `minio/cmd/admin-bucket-handlers.go` | `tests/minio/admin_bucket.rs` |
| ⬜ | MinIO | Admin | User admin APIs | `minio/cmd/admin-handlers-users.go` | `tests/minio/admin_users.rs` |
| ⬜ | MinIO | Admin | Config management | `minio/cmd/admin-handlers-config-kv.go` | `tests/minio/admin_config.rs` |
| ⬜ | MinIO | Admin | Heal operations | `minio/cmd/admin-heal-ops.go` | `tests/minio/admin_heal.rs` |
| ⬜ | MinIO | Admin | Server info | `minio/cmd/admin-server-info.go` | `tests/minio/admin_info.rs` |
| ⬜ | AWS S3 | Error | Invalid requests | `minio/cmd/api-errors.go` | `tests/minio/error_scenarios.rs`, `tests/aws/error_scenarios.rs` |
| ⬜ | AWS S3 | Error | Conflicts | `minio/cmd/api-errors.go` | `tests/minio/error_scenarios.rs`, `tests/aws/error_scenarios.rs` |
| ⬜ | AWS S3 | Error | Not found | `minio/cmd/api-errors.go` | `tests/minio/error_scenarios.rs`, `tests/aws/error_scenarios.rs` |
| ⬜ | AWS S3 | Error | Access denied | `minio/cmd/api-errors.go` | `tests/minio/error_access.rs` |
| ⬜ | AWS S3 | Error | Quota exceeded | `minio/cmd/api-errors.go` | `tests/minio/error_quota.rs` |
