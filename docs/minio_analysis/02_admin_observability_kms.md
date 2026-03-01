# 管理与观测协议分析（Admin / Health / Metrics / KMS）

## 1. Admin REST API

- 管理面前缀：`/minio/admin`
  - 证据：`minio/cmd/admin-router.go:31`
- 由 `registerAdminRouter` 统一注册，覆盖服务控制、用户策略、配置、副本、诊断等大量接口。
  - 证据：`minio/cmd/admin-router.go:138`, `:141`

协议特征：
- 传输层：HTTP/HTTPS
- 风格：REST + query 参数
- 鉴权：管理凭证/签名中间件（由 admin middleware 链处理）

## 2. Health Check 协议

- 健康检查前缀：`/minio/health`
  - 证据：`minio/cmd/healthcheck-router.go:32`
- 关键探针：
  - `/live`（liveness）
  - `/ready`（readiness）
  - `/cluster`、`/cluster/read`
  - 证据：`minio/cmd/healthcheck-router.go:41`, `:43`, `:47`, `:51`

协议特征：
- 传输层：HTTP/HTTPS
- 方法：GET/HEAD
- 用途：K8s/LB 探活与集群状态检查

## 3. Prometheus 指标协议

- 指标前缀位于 `/minio` 下，兼容多代路径：
  - 旧：`/prometheus/metrics`
  - V2：`/v2/metrics/cluster|bucket|node|resource`
  - V3：`/metrics/v3...`
  - 证据：`minio/cmd/metrics-router.go:29`, `:30`, `:31`, `:32`, `:33`, `:36`, `:64`, `:74`
- 文档明确使用 Prometheus pull model over HTTP/HTTPS。
  - 证据：`minio/docs/metrics/prometheus/README.md:3`, `:5`, `:6`

协议特征：
- 传输层：HTTP/HTTPS
- 格式：Prometheus exposition
- 鉴权模式：`jwt` 或 `public`
  - 证据：`minio/cmd/metrics-router.go:48`, `:49`, `:56`, `minio/docs/metrics/prometheus/README.md:39`

## 4. KMS API 协议

- 前缀：`/minio/kms`
  - 证据：`minio/cmd/kms-router.go:30`, `:40`
- 版本路由示例（`/v1`）：
  - `GET /status|/metrics|/apis|/version`
  - `POST /key/create`
  - `GET /key/list|/key/status`
  - 证据：`minio/cmd/kms-router.go:54`, `:55`, `:56`, `:57`, `:59`, `:60`, `:61`

协议特征：
- 传输层：HTTP/HTTPS
- 风格：REST
- 用途：KMS 状态观测与密钥管理操作

## 5. 结论

- MinIO 的运维协议栈基于 HTTP/HTTPS，拆分为四类：Admin、Health、Metrics、KMS。
- 其中 Metrics 明确兼容 Prometheus 采集模型；Health 与 Admin 满足云原生探活和自动化运维。
