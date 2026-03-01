# Amazon S3 API over HTTP/HTTPS 全量接口分析（MinIO 当前源码）

## 范围与结论

- 范围：`minio/cmd/api-router.go` 中 `registerAPIRouter()` 注册的所有 S3 HTTP/HTTPS 路由。
- 结论：
  - 核心接口全部走 HTTP 方法 + 路径 + Query 条件匹配。
  - 同时支持虚拟主机风格与 Path 风格桶路由。
  - 包含已实现接口与显式拒绝（NotImplemented）接口。

关键证据：
- `minio/cmd/api-router.go:254`
- `minio/cmd/api-router.go:262`
- `minio/cmd/api-router.go:286`
- `minio/cmd/api-router.go:289`

## 路由模型

## 1. 根路由

- `GET /?events=...` -> `ListenNotificationHandler`
- `GET /` -> `ListBucketsHandler`
- `GET //` -> `ListBucketsHandler`（兼容 S3 browser 双斜杠）

证据：`minio/cmd/api-router.go:632-644`

## 2. Bucket 寻址风格

- Virtual-host style: `https://{bucket}.{domain}/{object}`
- Path style: `https://{host}/{bucket}/{object}`

证据：`minio/cmd/api-router.go:265-289`

## 3. 资源级别

- Object 级路径：`/{bucket}/{object:.+}` 或 vhost 下 `/{object:.+}`
- Bucket 级路径：`/{bucket}` 或 vhost 下 `/`

## 接口清单（全量）

## A. Object 级接口

| API | Method | Path | Query/Header 条件 | Handler |
|---|---|---|---|---|
| HeadObject | `HEAD` | `/{object:.+}` | - | `HeadObjectHandler` |
| GetObjectAttributes | `GET` | `/{object:.+}` | `attributes=` | `GetObjectAttributesHandler` |
| CopyObjectPart | `PUT` | `/{object:.+}` | `partNumber` + `uploadId` + `x-amz-copy-source` | `CopyObjectPartHandler` |
| PutObjectPart | `PUT` | `/{object:.+}` | `partNumber` + `uploadId` | `PutObjectPartHandler` |
| ListObjectParts | `GET` | `/{object:.+}` | `uploadId` | `ListObjectPartsHandler` |
| CompleteMultipartUpload | `POST` | `/{object:.+}` | `uploadId` | `CompleteMultipartUploadHandler` |
| NewMultipartUpload | `POST` | `/{object:.+}` | `uploads=` | `NewMultipartUploadHandler` |
| AbortMultipartUpload | `DELETE` | `/{object:.+}` | `uploadId` | `AbortMultipartUploadHandler` |
| GetObjectACL (dummy) | `GET` | `/{object:.+}` | `acl=` | `GetObjectACLHandler` |
| PutObjectACL (dummy) | `PUT` | `/{object:.+}` | `acl=` | `PutObjectACLHandler` |
| GetObjectTagging | `GET` | `/{object:.+}` | `tagging=` | `GetObjectTaggingHandler` |
| PutObjectTagging | `PUT` | `/{object:.+}` | `tagging=` | `PutObjectTaggingHandler` |
| DeleteObjectTagging | `DELETE` | `/{object:.+}` | `tagging=` | `DeleteObjectTaggingHandler` |
| SelectObjectContent | `POST` | `/{object:.+}` | `select=` + `select-type=2` | `SelectObjectContentHandler` |
| GetObjectRetention | `GET` | `/{object:.+}` | `retention=` | `GetObjectRetentionHandler` |
| GetObjectLegalHold | `GET` | `/{object:.+}` | `legal-hold=` | `GetObjectLegalHoldHandler` |
| GetObject (Lambda ARN) | `GET` | `/{object:.+}` | `lambdaArn` | `GetObjectLambdaHandler` |
| GetObject | `GET` | `/{object:.+}` | - | `GetObjectHandler` |
| CopyObject | `PUT` | `/{object:.+}` | `x-amz-copy-source` | `CopyObjectHandler` |
| PutObjectRetention | `PUT` | `/{object:.+}` | `retention=` | `PutObjectRetentionHandler` |
| PutObjectLegalHold | `PUT` | `/{object:.+}` | `legal-hold=` | `PutObjectLegalHoldHandler` |
| PutObjectExtract (MinIO 扩展) | `PUT` | `/{object:.+}` | `x-amz-snowball-extract:true` | `PutObjectExtractHandler` |
| AppendObject (拒绝) | `PUT` | `/{object:.+}` | `x-amz-write-offset-bytes` | `errorResponseHandler` |
| PutObject | `PUT` | `/{object:.+}` | - | `PutObjectHandler` |
| DeleteObject | `DELETE` | `/{object:.+}` | - | `DeleteObjectHandler` |
| PostRestoreObject | `POST` | `/{object:.+}` | `restore=` | `PostRestoreObjectHandler` |

证据：`minio/cmd/api-router.go:300-408`

## B. Bucket 级接口

| API | Method | Bucket Path | Query 条件 | Handler |
|---|---|---|---|---|
| GetBucketLocation | `GET` | `/` | `location=` | `GetBucketLocationHandler` |
| GetBucketPolicy | `GET` | `/` | `policy=` | `GetBucketPolicyHandler` |
| GetBucketLifecycle | `GET` | `/` | `lifecycle=` | `GetBucketLifecycleHandler` |
| GetBucketEncryption | `GET` | `/` | `encryption=` | `GetBucketEncryptionHandler` |
| GetBucketObjectLockConfig | `GET` | `/` | `object-lock=` | `GetBucketObjectLockConfigHandler` |
| GetBucketReplicationConfig | `GET` | `/` | `replication=` | `GetBucketReplicationConfigHandler` |
| GetBucketVersioning | `GET` | `/` | `versioning=` | `GetBucketVersioningHandler` |
| GetBucketNotification | `GET` | `/` | `notification=` | `GetBucketNotificationHandler` |
| ListenNotification | `GET` | `/` | `events` | `ListenNotificationHandler` |
| ResetBucketReplicationStatus (MinIO 扩展) | `GET` | `/` | `replication-reset-status=` | `ResetBucketReplicationStatusHandler` |
| GetBucketACL (dummy) | `GET` | `/` | `acl=` | `GetBucketACLHandler` |
| PutBucketACL (dummy) | `PUT` | `/` | `acl=` | `PutBucketACLHandler` |
| GetBucketCors (dummy) | `GET` | `/` | `cors=` | `GetBucketCorsHandler` |
| PutBucketCors (dummy) | `PUT` | `/` | `cors=` | `PutBucketCorsHandler` |
| DeleteBucketCors (dummy) | `DELETE` | `/` | `cors=` | `DeleteBucketCorsHandler` |
| GetBucketWebsite (dummy) | `GET` | `/` | `website=` | `GetBucketWebsiteHandler` |
| GetBucketAccelerate (dummy) | `GET` | `/` | `accelerate=` | `GetBucketAccelerateHandler` |
| GetBucketRequestPayment (dummy) | `GET` | `/` | `requestPayment=` | `GetBucketRequestPaymentHandler` |
| GetBucketLogging (dummy) | `GET` | `/` | `logging=` | `GetBucketLoggingHandler` |
| GetBucketTagging | `GET` | `/` | `tagging=` | `GetBucketTaggingHandler` |
| DeleteBucketWebsite | `DELETE` | `/` | `website=` | `DeleteBucketWebsiteHandler` |
| DeleteBucketTagging | `DELETE` | `/` | `tagging=` | `DeleteBucketTaggingHandler` |
| ListMultipartUploads | `GET` | `/` | `uploads=` | `ListMultipartUploadsHandler` |
| ListObjectsV2M (MinIO 扩展) | `GET` | `/` | `list-type=2&metadata=true` | `ListObjectsV2MHandler` |
| ListObjectsV2 | `GET` | `/` | `list-type=2` | `ListObjectsV2Handler` |
| ListObjectVersionsM (MinIO 扩展) | `GET` | `/` | `versions=&metadata=true` | `ListObjectVersionsMHandler` |
| ListObjectVersions | `GET` | `/` | `versions=` | `ListObjectVersionsHandler` |
| GetBucketPolicyStatus | `GET` | `/` | `policyStatus=` | `GetBucketPolicyStatusHandler` |
| PutBucketLifecycle | `PUT` | `/` | `lifecycle=` | `PutBucketLifecycleHandler` |
| PutBucketReplicationConfig | `PUT` | `/` | `replication=` | `PutBucketReplicationConfigHandler` |
| PutBucketEncryption | `PUT` | `/` | `encryption=` | `PutBucketEncryptionHandler` |
| PutBucketPolicy | `PUT` | `/` | `policy=` | `PutBucketPolicyHandler` |
| PutBucketObjectLockConfig | `PUT` | `/` | `object-lock=` | `PutBucketObjectLockConfigHandler` |
| PutBucketTagging | `PUT` | `/` | `tagging=` | `PutBucketTaggingHandler` |
| PutBucketVersioning | `PUT` | `/` | `versioning=` | `PutBucketVersioningHandler` |
| PutBucketNotification | `PUT` | `/` | `notification=` | `PutBucketNotificationHandler` |
| ResetBucketReplicationStart (MinIO 扩展) | `PUT` | `/` | `replication-reset=` | `ResetBucketReplicationStartHandler` |
| PutBucket | `PUT` | `/` | - | `PutBucketHandler` |
| HeadBucket | `HEAD` | `/` | - | `HeadBucketHandler` |
| PostPolicy | `POST` | `/` | SigV4 Post Policy matcher | `PostPolicyBucketHandler` |
| DeleteMultipleObjects | `POST` | `/` | `delete=` | `DeleteMultipleObjectsHandler` |
| DeleteBucketPolicy | `DELETE` | `/` | `policy=` | `DeleteBucketPolicyHandler` |
| DeleteBucketReplication | `DELETE` | `/` | `replication=` | `DeleteBucketReplicationConfigHandler` |
| DeleteBucketLifecycle | `DELETE` | `/` | `lifecycle=` | `DeleteBucketLifecycleHandler` |
| DeleteBucketEncryption | `DELETE` | `/` | `encryption=` | `DeleteBucketEncryptionHandler` |
| DeleteBucket | `DELETE` | `/` | - | `DeleteBucketHandler` |
| GetBucketReplicationMetricsV2 (MinIO 扩展) | `GET` | `/` | `replication-metrics=2` | `GetBucketReplicationMetricsV2Handler` |
| GetBucketReplicationMetrics (deprecated) | `GET` | `/` | `replication-metrics=` | `GetBucketReplicationMetricsHandler` |
| ValidateBucketReplicationCreds | `GET` | `/` | `replication-check=` | `ValidateBucketReplicationCredsHandler` |
| ListObjectsV1 (legacy) | `GET` | `/` | 无特定 query | `ListObjectsV1Handler` |

证据：`minio/cmd/api-router.go:410-627`

## C. Root 级接口

| API | Method | Path | Query 条件 | Handler |
|---|---|---|---|---|
| ListenNotification | `GET` | `/` | `events` | `ListenNotificationHandler` |
| ListBuckets | `GET` | `/` | - | `ListBucketsHandler` |
| ListBuckets (兼容双斜杠) | `GET` | `//` | - | `ListBucketsHandler` |

证据：`minio/cmd/api-router.go:632-644`

## D. 显式拒绝（NotImplemented）接口

这些接口会命中 `notImplementedHandler`（返回 `ErrNotImplemented`），也是路由层“可匹配接口”。

## D1. Object 级 rejectedObjAPIs

| API 标识 | Method | Path | Query |
|---|---|---|---|
| `torrent` | `PUT`/`DELETE`/`GET` | `/{object:.+}` | `torrent=` |
| `acl` | `DELETE` | `/{object:.+}` | `acl=` |

证据：`minio/cmd/api-router.go:93-106`, `:293-298`

## D2. Bucket 级 rejectedBucketAPIs

| API 标识 | Method | Bucket Path | Query |
|---|---|---|---|
| `inventory` | `GET`/`PUT`/`DELETE` | `/` | `inventory=` |
| `cors` | `PUT`/`DELETE` | `/` | `cors=` |
| `metrics` | `GET`/`PUT`/`DELETE` | `/` | `metrics=` |
| `website` | `PUT` | `/` | `website=` |
| `logging` | `PUT`/`DELETE` | `/` | `logging=` |
| `accelerate` | `PUT`/`DELETE` | `/` | `accelerate=` |
| `requestPayment` | `PUT`/`DELETE` | `/` | `requestPayment=` |
| `acl` | `DELETE`/`PUT`/`HEAD` | `/` | `acl=` |
| `publicAccessBlock` | `DELETE`/`PUT`/`GET` | `/` | `publicAccessBlock=` |
| `ownershipControls` | `DELETE`/`PUT`/`GET` | `/` | `ownershipControls=` |
| `intelligent-tiering` | `DELETE`/`PUT`/`GET` | `/` | `intelligent-tiering=` |
| `analytics` | `DELETE`/`PUT`/`GET` | `/` | `analytics=` |

证据：`minio/cmd/api-router.go:108-168`, `:619-623`

## 方法覆盖统计

## 已实现 API 方法分布（registerAPIRouter 内显式注册）

- `GET`
- `PUT`
- `HEAD`
- `POST`
- `DELETE`

证据：`minio/cmd/api-router.go:300-644`

## 补充说明

- 文档仅基于“路由注册”层，不展开每个 handler 内部权限与参数细节。
- 实际请求还会经过 `s3APIMiddleware(...)` 鉴权、限流、追踪等中间件。
- 若同路径同方法存在多个 Query 条件分支，mux 按匹配条件区分。
