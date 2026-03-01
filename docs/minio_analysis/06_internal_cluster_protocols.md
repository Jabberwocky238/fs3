# 集群内部协议分析（节点间通信）

## 1. Internal Storage REST

- 前缀：`/minio/storage`
- 版本：`v63`
- 典型方法：`/health`, `/rfile`, `/rver`, `/rxl`, `/ls` 等

证据：
- `minio/cmd/storage-rest-common.go:21`, `:23`, `:27-44`
- `minio/cmd/storage-rest-server.go:1358-1372`

用途：
- 节点间磁盘读写、元数据处理、对象底层操作协同。

## 2. Internal Peer REST

- 前缀：`/minio/peer`
- 版本：`v39`
- 典型方法：`/health`, `/speedtest`, `/netperf`, `/verifybinary` 等

证据：
- `minio/cmd/peer-rest-common.go:22-25`
- `minio/cmd/peer-rest-server.go:1357-1367`

用途：
- 节点间管理与状态同步、性能测试、升级协同等。

## 3. Grid 协议（WebSocket 双向多路复用）

- 路由：
  - `/minio/grid/v1`
  - `/minio/grid/lock/v1`
- 连接机制：`http/https` 地址在 dial 时改写为 `ws/wss`

证据：
- `minio/internal/grid/manager.go:48`, `:51`
- `minio/internal/grid/grid.go:208`, `:209`
- `minio/cmd/routers.go:47`, `:50`

用途：
- 高并发双向 RPC/流式消息承载（含分布式锁、peer/storage handler）。

## 4. REST 与 Grid 的关系

当前实现是混合形态：

- 一部分能力仍保留 HTTP REST 路由（如 storage/peer 的部分方法）。
- 同时大量 handler 已经注册到 Grid（WebSocket）通道。

证据：
- Storage：`minio/cmd/storage-rest-server.go:1358-1372`（HTTP）与 `:1374-1396`（Grid 注册）
- Router：`minio/cmd/routers.go:35`, `:38`, `:47`, `:50`

## 5. 结论

- MinIO 内部协议并非单一 REST，而是“REST + WebSocket Grid”并存。
- 从演进方向看，Grid 是更核心的节点间通信骨干；REST 在特定路径与兼容层继续存在。
