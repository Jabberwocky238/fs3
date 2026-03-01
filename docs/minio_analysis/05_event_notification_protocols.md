# 事件通知协议分析

## 1. 总览

MinIO 的通知子系统在配置结构中显式包含 10 类目标协议/系统：

- AMQP
- Elasticsearch
- Kafka
- MQTT
- MySQL
- NATS
- NSQ
- PostgreSQL
- Redis
- Webhook

证据：`minio/internal/config/notify/config.go:26-36`

## 2. 各协议实现证据

| 目标 | 核心实现证据 | 备注 |
|---|---|---|
| AMQP | `minio/internal/event/target/amqp.go:104`, `:207` | 校验 URI，使用 `amqp091` 连接 |
| Kafka | `minio/internal/event/target/kafka.go:143`, `:408` | Kafka Version/SASL/TLS 配置 |
| MQTT | `minio/internal/event/target/mqtt.go:242`, `:250` | 使用 MQTT 客户端 options + broker |
| NATS | `minio/internal/event/target/nats.go:205`, `:66`, `:302` | 支持 NATS + JetStream，兼容旧 Streaming |
| NSQ | `minio/internal/event/target/nsq.go:130`, `:236` | `nsq.NewProducer` |
| Redis | `minio/internal/event/target/redis.go:335` | `redis.Dial("tcp", ...)` |
| MySQL | `minio/internal/event/target/mysql.go:119`, `:364` | DSN 校验 + MySQL 驱动 |
| PostgreSQL | `minio/internal/event/target/postgresql.go:366` | `sql.Open("postgres", ...)` |
| Elasticsearch | `minio/internal/event/target/elasticsearch.go:412` | ES v7 client |
| Webhook | `minio/internal/event/target/webhook.go:177` | HTTP POST 事件推送 |

## 3. 传输与安全模式

- Webhook/Elasticsearch：HTTP/HTTPS URL，配置解析阶段使用 `ParseHTTPURL`。
  - 证据：`minio/internal/config/notify/parse.go:1462`, `:1569`
- Kafka/NATS/NSQ 支持 TLS/SASL 等连接参数。
  - 证据：`minio/internal/event/target/kafka.go:50-57`, `:354-369`; `minio/internal/event/target/nats.go:49-51`, `:191-197`; `minio/internal/event/target/nsq.go:43-44`, `:228-231`
- MySQL/PostgreSQL/Redis 走数据库/缓存原生协议。

## 4. 与 S3 事件 API 的关系

- S3 API 路由包含 bucket notification 配置接口与 `ListenNotification` 实时监听接口。
  - 证据：`minio/cmd/api-router.go:443`, `:446`, `:561`, `:634`
- 这些接口定义“事件源与过滤规则”；目标协议决定“事件投递落点”。

## 5. 结论

- MinIO 的事件协议兼容面非常广，覆盖消息队列、数据库、搜索引擎与通用 Webhook。
- 对接时应按目标系统选择协议，并在 TLS/认证参数上做显式配置。
