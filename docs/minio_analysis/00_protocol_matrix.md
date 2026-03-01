# 协议兼容矩阵

> 基于当前源码快照（`./minio`）整理。

## 1. 对外数据访问协议

| 协议 | 兼容/用途 | 关键证据 |
|---|---|---|
| Amazon S3 API over HTTP/HTTPS | MinIO 核心对象存储接口，声明为 S3 兼容 | `minio/README.md:16`, `minio/README.md:19`, `minio/cmd/api-router.go:254` |
| S3 Auth: Signature V4 | S3 主认证机制 | `minio/cmd/signature-v4.go`, `minio/cmd/auth-handler.go:596` |
| S3 Auth: Signature V2（兼容） | 兼容旧客户端，含 presign 校验 | `minio/cmd/signature-v2.go:294`, `minio/cmd/auth-handler.go:186` |
| S3 Streaming SigV4（aws-chunked） | 流式分块上传签名 | `minio/cmd/streaming-signature-v4.go:43`, `minio/cmd/streaming-signature-v4.go:97` |

## 2. 安全令牌与身份相关协议

| 协议 | 兼容/用途 | 关键证据 |
|---|---|---|
| AWS STS Query API（2011-06-15） | 临时凭证签发 | `minio/cmd/sts-handlers.go:47`, `minio/docs/sts/README.md:3` |
| STS 扩展：WebIdentity / ClientGrants / LDAP / Certificate / CustomToken | 多身份源换取临时凭证 | `minio/cmd/sts-handlers.go:161`, `:168`, `:174`, `:181`, `:186` |
| OpenID Connect (OIDC) | Web 身份联合登录/令牌验证 | `minio/docs/sts/README.md:18`, `minio/internal/config/identity/openid/help.go:37` |
| LDAP / AD | 目录认证与 STS 联合 | `minio/docs/sts/README.md:19`, `minio/internal/config/identity/ldap/help.go:31` |
| Identity Management Plugin（Webhook） | 自定义 token 鉴权（CustomToken） | `minio/docs/iam/identity-management-plugin.md:5`, `:32` |
| OPA HTTP API | 外部策略引擎授权 | `minio/docs/iam/opa.md:3`, `minio/internal/config/policy/opa/config.go:138` |

## 3. 运维与观测协议

| 协议 | 兼容/用途 | 关键证据 |
|---|---|---|
| Admin REST API | 管理面接口（用户/策略/集群/副本等） | `minio/cmd/admin-router.go:31`, `:141` |
| Health Check over HTTP | 存活/就绪/集群健康 | `minio/cmd/healthcheck-router.go:32`, `:41`, `:47`, `:51` |
| Prometheus Exposition（HTTP/HTTPS） | `/minio/v2/metrics/*` + `/minio/metrics/v3` | `minio/cmd/metrics-router.go:29`, `:30`, `:36`, `:64`, `:74` |
| KMS REST API | `/minio/kms/v1/*` 密钥管理接口 | `minio/cmd/kms-router.go:30`, `:54`, `:59` |

## 4. 文件传输协议

| 协议 | 兼容/用途 | 关键证据 |
|---|---|---|
| FTP | 明文 FTP（不推荐） | `minio/docs/ftp/README.md:26`, `minio/cmd/ftp-server.go:71` |
| FTPS (FTP over TLS) | FTP 显式 TLS/强制 TLS | `minio/docs/ftp/README.md:24`, `minio/cmd/ftp-server.go:152`, `:157` |
| SFTP (SSH File Transfer Protocol) | SSH 子系统文件传输 | `minio/docs/ftp/README.md:20`, `minio/cmd/sftp-server.go:394` |

## 5. 事件通知目标协议

| 协议 | 兼容/用途 | 关键证据 |
|---|---|---|
| AMQP | 事件投递到 RabbitMQ 等 | `minio/internal/config/notify/config.go:27`, `minio/internal/event/target/amqp.go:207` |
| Kafka | 事件投递到 Kafka | `minio/internal/config/notify/config.go:29`, `minio/internal/event/target/kafka.go:408` |
| MQTT | 事件投递到 MQTT Broker | `minio/internal/config/notify/config.go:30`, `minio/internal/event/target/mqtt.go:242` |
| NATS / JetStream /（兼容）Streaming | 事件投递到 NATS 生态 | `minio/internal/config/notify/config.go:32`, `minio/internal/event/target/nats.go:205`, `:66` |
| NSQ | 事件投递到 NSQ | `minio/internal/config/notify/config.go:33`, `minio/internal/event/target/nsq.go:130` |
| Redis | 事件写入 Redis | `minio/internal/config/notify/config.go:35`, `minio/internal/event/target/redis.go:335` |
| MySQL | 事件写入 MySQL | `minio/internal/config/notify/config.go:31`, `minio/internal/event/target/mysql.go:364` |
| PostgreSQL | 事件写入 PostgreSQL | `minio/internal/config/notify/config.go:34`, `minio/internal/event/target/postgresql.go:366` |
| Elasticsearch | 事件写入 ES | `minio/internal/config/notify/config.go:28`, `minio/internal/event/target/elasticsearch.go:412` |
| Webhook (HTTP/HTTPS) | 事件 POST 回调 | `minio/internal/config/notify/config.go:36`, `minio/internal/event/target/webhook.go:177` |

## 6. 集群内部协议

| 协议 | 兼容/用途 | 关键证据 |
|---|---|---|
| Storage REST API | 节点间磁盘/对象元数据 RPC | `minio/cmd/storage-rest-common.go:21`, `:23`, `minio/cmd/storage-rest-server.go:1358` |
| Peer REST API | 节点间管理与状态 RPC | `minio/cmd/peer-rest-common.go:22`, `:24`, `minio/cmd/peer-rest-server.go:1357` |
| Grid over WebSocket (`ws/wss`) | 节点间双向多路复用通信 | `minio/internal/grid/manager.go:48`, `:51`, `minio/internal/grid/grid.go:208`, `:209` |
| Lock Grid 子路由 | 分布式锁通道 | `minio/internal/grid/manager.go:51`, `minio/cmd/routers.go:47` |

## 备注

- “兼容”指源码中显式实现/路由/驱动存在，不等同于默认开启。
- FTP/SFTP、通知目标、身份插件等通常需要额外配置才会生效。
