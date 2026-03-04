# MinIO测试可被AWS SDK覆盖分析

## 已复制到tests/aws/的测试 ✅ (15个)
1. bucket.rs - 桶基本操作
2. bucket_config.rs - 桶配置
3. conditional.rs - 条件请求 (if-match, if-none-match)
4. error_scenarios.rs - 错误场景测试
5. list_objects.rs - 列举对象
6. multipart.rs - 分片上传
7. object.rs - 对象操作
8. object_advanced.rs - 高级对象操作 (tagging)
9. object_features.rs - 对象特性
10. object_lock.rs - 对象锁定
11. policy.rs - 桶策略
12. policy_advanced.rs - 高级策略 (Deny优先级, 通配符)
13. presigned.rs - 预签名URL
14. versioning.rs - 版本控制
15. website.rs - 网站配置

## 可以复制的MinIO测试（使用create_minio_client）

### 高优先级 - 核心功能 ✅ 已全部复制
- ✅ **list_objects.rs** - 列举对象（递归/前缀）
- ✅ **object_advanced.rs** - 高级对象操作
- ✅ **object_features.rs** - 对象特性
- ✅ **object_lock.rs** - 对象锁定
- ✅ **policy.rs** - 桶策略
- ✅ **policy_advanced.rs** - 高级策略
- ✅ **conditional.rs** - 条件请求
- ✅ **error_scenarios.rs** - 错误场景

### 中优先级 - 扩展功能
- **select.rs** - S3 Select查询
- **post_policy.rs** - POST策略
- **batch_version.rs** - 批量版本操作
- **list_advanced.rs** - 高级列举
- **object_lock_enforcement.rs** - 对象锁定强制
- **versioning_enforcement.rs** - 版本控制强制

### 低优先级 - 特殊场景
- **content_md5.rs** - MD5校验

## 不适合用AWS SDK的测试（MinIO特有或需要特殊环境）
- accelerate.rs - 使用setup_client（未定义）
- analytics.rs - 使用setup_client（未定义）
- auth_v4.rs - 使用setup_client（未定义）
- cors.rs - 使用setup_client（未定义）
- cors_enforcement.rs - 使用setup_client（未定义）
- distributed.rs - 分布式特性
- error_access.rs - 访问错误
- error_quota.rs - 配额错误
- gateway_minio.rs - MinIO网关
- gateway_s3.rs - S3网关
- health.rs - 健康检查
- inventory.rs - 清单
- lifecycle_execution.rs - 生命周期执行
- logging.rs - 日志配置
- metadata_k8s.rs - K8s元数据
- metadata_postgres.rs - Postgres元数据
- metrics.rs - 指标
- metrics_config.rs - 指标配置
- notifications.rs - 通知
- rate_limit.rs - 速率限制
- replication_sync.rs - 复制同步
- request_payment.rs - 请求付款
- restore.rs - 恢复
- sse_c.rs - SSE-C加密
- sse_kms.rs - SSE-KMS加密
- sse_s3.rs - SSE-S3加密
- tls.rs - TLS

## 总结
- ✅ 已复制: 15个 (核心功能全覆盖)
- 可选复制（中优先级）: 6个
- 可选复制（低优先级）: 1个
- 不适合复制: 29个

**状态**: 所有高优先级核心S3功能测试已完成AWS SDK版本
