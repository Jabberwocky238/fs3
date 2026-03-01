# 08 S3 存储后端实现要求（基于 MinIO 源码复查）

## 1. 目标

本文聚焦一个问题：要实现与 MinIO S3 语义兼容的存储后端，底层必须提供哪些接口、类型和策略能力。

本结论不是按 HTTP 路由层展开（07 已覆盖），而是按存储内核契约展开。

## 2. 复查源码范围（关键锚点）

- 对象层主接口：`minio/cmd/object-api-interface.go:246` (`type ObjectLayer interface`)
- 存储层主接口：`minio/cmd/storage-interface.go:29` (`type StorageAPI interface`)
- 核心数据类型：`minio/cmd/object-api-datatypes.go:108` (`BucketInfo`), `:121` (`ObjectInfo`)
- 请求选项类型：`minio/cmd/object-api-interface.go:63` (`ObjectOptions`), `:170` (`MakeBucketOptions`), `:179` (`DeleteBucketOptions`), `:187` (`BucketOptions`)
- Bucket 元数据聚合：`minio/cmd/bucket-metadata.go:68` (`BucketMetadata`)
- Bucket 元数据查询入口：`minio/cmd/bucket-metadata-sys.go:259-422`
- 治理/策略配置类型：
  - versioning: `minio/internal/bucket/versioning/versioning.go:50`
  - lifecycle: `minio/internal/bucket/lifecycle/lifecycle.go:103`, `rule.go:35`
  - replication: `minio/internal/bucket/replication/replication.go:40`, `rule.go:131`
  - bucket SSE: `minio/internal/bucket/encryption/bucket-sse-config.go:79`
  - object lock: `minio/internal/bucket/object/lock/lock.go:155,232,365,530`
  - notification: `minio/internal/event/config.go:238`
  - quota: `minio/cmd/bucket-quota.go:33,87`
- 策略动作校验入口：
  - `minio/cmd/bucket-policy-handlers.go:57,142,185`
  - `minio/cmd/object-handlers.go`
  - `minio/cmd/object-multipart-handlers.go`

## 3. 后端必须实现的接口契约

### 3.1 对象层（`ObjectLayer`）

`ObjectLayer` 是 S3 语义的核心抽象，建议在 Rust 中作为主 trait。

能力分组如下：

1. Bucket 管理
- `MakeBucket`
- `GetBucketInfo`
- `ListBuckets`
- `DeleteBucket`
- `ListObjects` / `ListObjectsV2` / `ListObjectVersions`
- `Walk`（包含版本与 delete-marker）

2. Object 读写
- `GetObjectNInfo`
- `GetObjectInfo`
- `PutObject`
- `CopyObject`
- `DeleteObject`
- `DeleteObjects`
- `TransitionObject` / `RestoreTransitionedObject`

3. Multipart
- `ListMultipartUploads`
- `NewMultipartUpload`
- `CopyObjectPart`
- `PutObjectPart`
- `GetMultipartInfo`
- `ListObjectParts`
- `AbortMultipartUpload`
- `CompleteMultipartUpload`

4. 元数据/标签
- `PutObjectMetadata`
- `PutObjectTags`
- `GetObjectTags`
- `DeleteObjectTags`

5. 健康/修复/扫描
- `NSScanner`
- `HealFormat` / `HealBucket` / `HealObject` / `HealObjects`
- `CheckAbandonedParts`
- `Health`

6. 部署拓扑相关
- `GetDisks`
- `SetDriveCounts`
- `StorageInfo` / `LocalStorageInfo`
- `BackendInfo`
- `Legacy`

### 3.2 存储层（`StorageAPI`）

`StorageAPI` 是磁盘/节点粒度接口，覆盖卷、文件、版本元数据、读写流、重命名和批量删除等。

关键能力：

1. 盘状态与身份
- `IsOnline`, `LastConn`, `IsLocal`, `Hostname`, `Endpoint`
- `GetDiskID`, `SetDiskID`, `DiskInfo`, `Healing`

2. 卷（Volume）管理
- `MakeVol`, `MakeVolBulk`, `ListVols`, `StatVol`, `DeleteVol`

3. 元数据版本操作
- `WriteMetadata`, `UpdateMetadata`, `ReadVersion`, `ReadXL`
- `DeleteVersion`, `DeleteVersions`, `DeleteBulk`
- `RenameData`

4. 文件与对象分片操作
- `ReadFile`, `AppendFile`, `CreateFile`, `ReadFileStream`
- `RenameFile`, `RenamePart`, `CheckParts`, `VerifyFile`
- `ReadParts`, `ReadMultiple`, `CleanAbandonedData`
- `WriteAll`, `ReadAll`

5. 列举与扫描
- `ListDir`, `WalkDir`, `NSScanner`

结论：如果只做单机最小实现，可先实现 `ObjectLayer` 的最小子集并以内嵌文件系统替代完整 `StorageAPI`；但要做 MinIO 级语义，最终必须具备 version-aware metadata + multipart + scanner/heal 能力。

## 4. 必需核心类型（bucket / object）

### 4.1 Bucket 类型要求（`BucketInfo` + bucket feature）

从 `BucketInfo` 可知至少需要：
- `Name`
- `Created`
- `Deleted`（站点复制/删除跟踪）
- `Versioning`（bucket 级）
- `ObjectLocking`（bucket 级）

对应结论：bucket 不是只有名字，必须带“能力位”（versioning/object-lock）。

### 4.2 Object 类型要求（`ObjectInfo`）

`ObjectInfo` 显示对象至少要支持以下维度：

1. 基础标识
- `Bucket`, `Name`, `ModTime`, `Size`, `ETag`, `ContentType`, `ContentEncoding`

2. 版本与删除标记
- `VersionID`, `IsLatest`, `DeleteMarker`, `NumVersions`, `SuccessorModTime`

3. 用户元数据与标签
- `UserDefined`, `UserTags`

4. 分片与校验
- `Parts[]`, `Checksum`, `DataBlocks`, `ParityBlocks`

5. 生命周期/归档/恢复
- `TransitionedObject`, `RestoreExpires`, `RestoreOngoing`, `Expires`, `StorageClass`

6. 复制状态
- `ReplicationStatus`, `ReplicationStatusInternal`
- `VersionPurgeStatus`, `VersionPurgeStatusInternal`

结论：若仅用“文件 + size + etag”模型，无法支持版本、对象锁、复制和 restore。

### 4.3 请求级行为控制（`ObjectOptions` / bucket options）

`ObjectOptions` 在 MinIO 里承担跨 API 的语义开关：
- 版本语义：`Versioned`, `VersionSuspended`, `VersionID`
- 条件请求：`CheckPrecondFn`, `HasIfMatch`
- 加密与校验：`ServerSideEncryption`, `WantChecksum`
- 复制/删除复制：`DeleteReplication`, `ReplicationRequest`
- 保留与法务：`EvalRetentionBypassFn`
- 标签/属性请求：`Tagging`, `ObjectAttributes`, `MaxParts`, `PartNumberMarker`

bucket 选项同样关键：
- `MakeBucketOptions`（锁、版本、强制创建、跨站复制创建时间）
- `DeleteBucketOptions`（force、no-recreate、site-replication delete op）
- `BucketOptions`（deleted/cached/no-metadata）

## 5. Bucket 策略与治理配置模型

### 5.1 聚合模型：`BucketMetadata`

`BucketMetadata` 是 bucket 治理配置的“聚合根”，包含：
- 原始配置载荷（XML/JSON）
- 各配置的 `UpdatedAt`
- 已解析的结构化配置指针（policy/lifecycle/versioning/lock/...）

这意味着后端设计不能只存“配置文件”，还要有：
- 解析后对象缓存
- 配置更新时间戳
- 配置一致性加载流程

### 5.2 配置子系统（按 MinIO 当前结构）

1. 访问策略
- Bucket policy（允许/拒绝 action + resource + condition）

2. 版本治理
- Versioning（Enabled/Suspended 语义）

3. 生命周期
- Lifecycle + Rule（过期、转储、非当前版本清理等）

4. 跨桶/跨站复制
- Replication Config + Rule

5. 加密
- Bucket SSE config

6. 对象锁
- Bucket object-lock config
- Object retention / legal-hold

7. 事件通知
- Notification config（事件到 target）

8. Tagging / Quota / Targets
- Bucket tags
- Bucket quota
- Bucket replication targets 元数据

`BucketMetadataSys` 的 getter（`GetVersioningConfig` / `GetBucketPolicy` / `GetObjectLockConfig` / `GetLifecycleConfig` / `GetNotificationConfig` / `GetSSEConfig` / `GetQuotaConfig` / `GetReplicationConfig` / `GetBucketTargetsConfig`）已经体现该拆分。

## 6. 权限动作面（Policy Action Surface）

从 handler 调用可见：S3 API 最终映射为大量 `policy.*Action` 检查，并由以下入口执行：
- `checkRequestAuthType(...)`
- `authenticateRequest(...)`
- `authorizeRequest(...)`

代表性动作组：

1. Bucket 级
- `CreateBucketAction`, `DeleteBucketAction`, `ForceDeleteBucketAction`
- `ListBucketAction`, `HeadBucketAction`, `ListAllMyBucketsAction`
- `GetBucketLocationAction`
- `PutBucketPolicyAction`, `GetBucketPolicyAction`, `DeleteBucketPolicyAction`
- `PutBucketTaggingAction`, `GetBucketTaggingAction`
- `PutBucketObjectLockConfigurationAction`, `GetBucketObjectLockConfigurationAction`

2. Object 级
- `GetObjectAction`, `PutObjectAction`, `DeleteObjectAction`
- `GetObjectTaggingAction`, `PutObjectTaggingAction`, `DeleteObjectTaggingAction`
- `GetObjectRetentionAction`, `PutObjectRetentionAction`
- `GetObjectLegalHoldAction`, `PutObjectLegalHoldAction`
- `RestoreObjectAction`, `ReplicateObjectAction`, `ReplicateDeleteAction`

3. Multipart 级
- `AbortMultipartUploadAction`
- `ListMultipartUploadPartsAction`
- （创建/上传分片/完成通常依附 `PutObjectAction`）

结论：协议兼容不仅是接口存在，还要求 action 粒度的授权模型。

## 7. 对当前 Rust 工程（`src/types` / `src/types/traits`）的落地建议

### 7.1 最小可用（先跑通 S3 主链路）

必须先有：
- bucket/object/multipart 的核心类型（带 version 字段）
- 对象 CRUD + multipart + list（v1/v2/version）
- bucket policy 基础动作校验（至少覆盖 list/get/put/delete）

可暂缓：
- scanner/heal
- decommission/rebalance 相关开关
- 完整 replication status 矩阵

### 7.2 结构建议

1. `src/types/s3/`
- `request.rs`: 按 API 建请求体（含 query/header 解析后语义字段）
- `response.rs`: 按 API 建响应体
- `mod.rs`: 统一导出（优先 `pub use ...::*`）

2. `src/types/traits/s3_handler.rs`
- 按四类接口拆 trait：
  - bucket handlers
  - object handlers
  - multipart handlers
  - bucket-config handlers（policy/lifecycle/versioning/encryption/lock/notification/tagging/replication）

3. 存储后端 trait（建议新增）
- `S3ObjectLayer`（对齐 `ObjectLayer`）
- 可选 `StorageBackend`（后续向 `StorageAPI` 靠拢）

## 8. 实现检查清单（用于 08 之后编码）

- bucket 是否具备 versioning/object-lock 能力位
- object 是否具备 versionID/delete-marker/user-metadata/tags/parts/checksum 字段
- multipart 生命周期是否完整（new/upload/list/complete/abort）
- policy action 是否细分到 bucket/object/multipart 操作
- bucket 元配置是否统一归档并带更新时间
- retention/legal-hold 是否在对象写入/删除链路生效

---

如果后续你要我继续，我可以按这个 08 文档直接给出 `Rust trait + struct` 的最小实现骨架（与 `07_s3_http_https_all_interfaces_methods` 一一绑定）。
