# MinIO 远程盘 storageREST 实现梳理

本文档只讨论 MinIO 的“远程盘”实现，即 `storageRESTClient` / `storageRESTServer` 这一层如何把远端节点上的本地盘包装成统一的 `StorageAPI`。

目标：

1. 说明 MinIO 是否真的区分本地盘和远程盘
2. 说明远程盘是另一种存储格式，还是另一种访问路径
3. 说明 `PutObject` 关键写入链路在远程盘上的真实走向

参考源码：

- `minio/cmd/object-api-common.go`
- `minio/cmd/storage-interface.go`
- `minio/cmd/storage-rest-client.go`
- `minio/cmd/storage-rest-server.go`
- `minio/cmd/storage-rest-common.go`
- `minio/cmd/xl-storage-disk-id-check.go`
- `minio/cmd/xl-storage.go`
- `minio/cmd/erasure-sets.go`

## 1. 总结先行

MinIO 在存储模型上确实区分：

1. 本地盘
2. 远程盘

但这个区分不是“本地盘一种格式，远程盘另一种格式”，而是：

1. 本地盘：直接调用本机 `xlStorage`
2. 远程盘：通过 `storageRESTClient` 发给远端节点
3. 远端节点再由 `storageRESTServer` 调用该节点自己的本地 `xlStorage`

因此：

1. 远程盘不是第三种独立后端
2. 远程盘不是另一套 `xl.meta` / data-dir 语义
3. 远程盘只是把“另一台机器上的本地盘”包装成统一 `StorageAPI`

更准确地说，MinIO 的远程盘是“远程访问本地盘”，不是“远程盘专属存储格式”。

## 2. StorageAPI 是统一抽象

文件：`minio/cmd/storage-interface.go`

`StorageAPI` 同时承载：

1. 盘身份信息：`IsOnline`、`IsLocal`、`Hostname`、`Endpoint`、`GetDiskID`
2. 元数据操作：`ReadVersion`、`WriteMetadata`、`RenameData`
3. 文件操作：`CreateFile`、`ReadFile`、`RenameFile`、`Delete`
4. 小数据直接写：`WriteAll`

其中关键点是：

- `IsLocal()` 明确区分本地盘和远程盘
- 上层对象层只依赖 `StorageAPI`
- 上层不关心底下到底是本地调用还是远程 RPC

这说明 MinIO 的盘抽象在接口层已经把“本地/远程”统一掉了。

## 3. 本地盘与远程盘的创建分流

文件：`minio/cmd/object-api-common.go`

```go
61:func newStorageAPI(endpoint Endpoint, opts storageOpts) (storage StorageAPI, err error) {
63:    storage, err := newXLStorage(endpoint, opts.cleanUp)
67:    return newXLStorageDiskIDCheck(storage, opts.healthCheck), nil
70:    return newStorageRESTClient(endpoint, opts.healthCheck, globalGrid.Load())
}
```

这里是最关键的分流点：

1. `endpoint.IsLocal == true`
   返回 `newXLStorage(...)` 再包一层 `newXLStorageDiskIDCheck(...)`
2. `endpoint.IsLocal == false`
   返回 `newStorageRESTClient(...)`

所以 MinIO 确实区分了两种盘访问路径：

1. 本地盘：`xlStorageDiskIDCheck -> xlStorage`
2. 远程盘：`storageRESTClient -> 远端 storageRESTServer -> 远端 xlStorageDiskIDCheck -> 远端 xlStorage`

## 4. 远程盘客户端：storageRESTClient

文件：`minio/cmd/storage-rest-client.go`

`storageRESTClient` 的定义非常直接：

```go
162:type storageRESTClient struct {
163:    endpoint   Endpoint
164:    restClient *rest.Client
165:    gridConn   *grid.Subroute
166:    diskID     atomic.Pointer[string]
}
```

这说明远程盘客户端同时持有两类通信通道：

1. `restClient`
2. `gridConn`

也就是说 MinIO 远程盘不是只靠一种协议。

### 4.1 远程盘客户端的两个通信面

从实现上看，MinIO 远程盘至少用了两种通信方式：

1. 基于 HTTP REST 的请求
2. 基于 `grid` 的 RPC / stream

典型例子：

- `CreateFile()` 走 HTTP POST
- `ReadVersion()` 在不读 data 时走 `grid`
- `RenameData()` 走 `grid`
- `WriteAll()` 走 `grid`

所以不能把 MinIO 远程盘简单理解为“纯 HTTP 文件接口”。

### 4.2 客户端如何统一错误

文件：`minio/cmd/storage-rest-client.go`

`toStorageErr()` 的作用是把网络层错误转换回本地存储错误语义，例如：

1. 网络断开映射到 `errDiskNotFound`
2. 远端返回的字符串错误映射回 `errFileNotFound`、`errFaultyDisk` 等

这一步很重要，因为上层对象层期望看到的是统一的 `StorageAPI` 错误，而不是裸网络错误。

结论：

- `storageRESTClient` 不只是转发请求
- 它还负责把“网络错误”折叠成“磁盘错误”

## 5. 远程盘服务端：storageRESTServer

文件：`minio/cmd/storage-rest-server.go`

`storageRESTServer` 很薄：

```go
55:type storageRESTServer struct {
56:    endpoint Endpoint
}
```

它本身不直接持有底层磁盘对象，而是通过 endpoint 反查：

```go
88:func (s *storageRESTServer) getStorage() StorageAPI {
89:    return getStorageViaEndpoint(s.endpoint)
}
```

也就是说：

1. 每个本地 endpoint 都会注册一个 `storageRESTServer`
2. 请求进入服务端后，会根据 endpoint 找到本机对应的 `StorageAPI`
3. 这个 `StorageAPI` 实际上是本地 `xlStorageDiskIDCheck`

## 6. 远程盘服务端只暴露本地 endpoint

文件：`minio/cmd/storage-rest-server.go`

`registerStorageRESTHandlers()` 里只为 `endpoint.IsLocal` 的 endpoint 注册服务端：

```go
1350:        for _, endpoint := range serverPool.Endpoints {
1351:            if !endpoint.IsLocal {
1352:                continue
1353:            }
```

含义非常明确：

1. 只有本机实际拥有的盘，才会在本机暴露 storage REST 服务
2. 远程盘客户端访问的是“另一台机器暴露的本地盘服务”

所以 MinIO 的远程盘不是单独部署的“remote disk daemon”，而是 MinIO 节点自己把本地盘挂到 internode API 上。

## 7. 远程盘服务端的底层仍是 xlStorageDiskIDCheck

同文件中，本地盘注册时实际创建的是：

```go
1400:                xl, err := newXLStorage(endpoint, false)
1410:                storage := newXLStorageDiskIDCheck(xl, true)
```

然后塞入：

1. `globalLocalDrivesMap`
2. `globalLocalSetDrives`

这说明远程请求最终落到的并不是另一套后端，而仍是：

1. `xlStorageDiskIDCheck`
2. `xlStorage`

因此远程盘最终执行的文件系统逻辑和本地盘是一致的。

## 8. 远程盘的鉴权与磁盘身份校验

文件：`minio/cmd/storage-rest-server.go`

服务端请求进入后不是直接执行，而要先过两层校验：

1. `IsAuthValid()`
2. `IsValid()`

### 8.1 鉴权

`IsAuthValid()` 依赖：

1. JWT 风格的 internode token
2. `X-Minio-Time` 的时钟偏移校验

这说明 storage REST 不是裸开放接口，而是 MinIO 节点间内部 API。

### 8.2 磁盘身份校验

`IsValid()` 会校验请求携带的 `diskID` 是否与当前底层磁盘一致。

如果不一致，返回：

- `errDiskStale`

这一步非常关键，因为远程客户端缓存的是“某个 endpoint 对应的某块盘”，而不是“只要这个 URL 活着就算同一块盘”。

结论：

远程盘访问不只是“访问远端地址”，还要保证：

1. 访问的是合法 MinIO 节点
2. 访问的是预期那块磁盘，而不是被替换后的另一块盘

## 9. 本地盘包装层：xlStorageDiskIDCheck 的角色

文件：`minio/cmd/xl-storage-disk-id-check.go`

`xlStorageDiskIDCheck` 不是另一种盘类型，而是本地 `xlStorage` 的包装层。

它主要负责：

1. disk-id 一致性检查
2. health tracking
3. timeout / deadline 包装
4. metrics 统计
5. 某些调用的读写计数更新

关键函数：

```go
437:func (p *xlStorageDiskIDCheck) CreateFile(...)
483:func (p *xlStorageDiskIDCheck) RenameData(...)
```

所以无论是：

1. 本地对象层直接调用本地盘
2. 远程节点通过 storage REST 调用这块盘

最终都通常会经过这层包装。

## 10. `CreateFile` 在远程盘上的真实调用链

### 10.1 客户端侧

文件：`minio/cmd/storage-rest-client.go`

```go
392:func (client *storageRESTClient) CreateFile(ctx context.Context, origvolume, volume, path string, size int64, reader io.Reader) error {
399:    respBody, err := client.call(ctx, storageRESTMethodCreateFile, values, io.NopCloser(reader), size)
404:    _, err = waitForHTTPResponse(respBody)
}
```

说明：

1. `CreateFile` 通过 HTTP POST 把流直接发给远端
2. body 不是先完整缓冲后再发
3. 服务端可能长时间处理，因此客户端用 `waitForHTTPResponse()` 等待完成信号

### 10.2 服务端侧

文件：`minio/cmd/storage-rest-server.go`

```go
336:func (s *storageRESTServer) CreateFileHandler(w http.ResponseWriter, r *http.Request) {
353:    done(s.getStorage().CreateFile(r.Context(), origvolume, volume, filePath, int64(fileSize), body))
}
```

说明：

1. 服务端从 HTTP body 读取流
2. 直接调用本地 `StorageAPI.CreateFile`
3. 这里的 `StorageAPI` 是本地 `xlStorageDiskIDCheck`

### 10.3 本地最终落盘

接下来的路径与本地盘相同：

1. `xlStorageDiskIDCheck.CreateFile()`
2. `xlStorage.CreateFile()`
3. `xlStorage.writeAllDirect()`

结论：

- 远程盘的 `CreateFile` 只是多了一跳网络转发
- 最终写盘语义仍然是远端节点本地 `xlStorage` 的语义

## 11. `RenameData` 在远程盘上的真实调用链

`RenameData` 与 `CreateFile` 不同，它不走简单 HTTP body 上传，而是走 `grid` RPC。

### 11.1 客户端侧

文件：`minio/cmd/storage-rest-client.go`

```go
485:func (client *storageRESTClient) RenameData(ctx context.Context, srcVolume, srcPath string, fi FileInfo,
488:    params := RenameDataHandlerParams{
499:    resp, err = storageRenameDataRPC.Call(ctx, client.gridConn, &params)
```

如果 `fi.Data != nil`，则走：

- `storageRenameDataInlineRPC`

这说明 `RenameData` 不是靠 URL query + HTTP body 硬编码传参，而是走结构化 RPC。

### 11.2 服务端侧

文件：`minio/cmd/storage-rest-server.go`

```go
703:func (s *storageRESTServer) RenameDataHandler(p *RenameDataHandlerParams) (*RenameDataResp, *grid.RemoteErr) {
708:    resp, err := s.getStorage().RenameData(context.Background(), p.SrcVolume, p.SrcPath, p.FI, p.DstVolume, p.DstPath, p.Opts)
}
```

结论：

1. `RenameData` 由远端服务端直接调用本地 `StorageAPI.RenameData`
2. 远端 `RenameData` 不改变提交顺序
3. 真正提交语义仍由远端 `xlStorage.RenameData()` 决定

## 12. 为什么有些调用走 HTTP，有些走 grid

从代码现状看，可以观察到一个倾向：

1. 需要原始流 body 的操作，常走 HTTP handler
2. 参数结构化、返回结构化的操作，常走 `grid` handler

例如：

- `CreateFile`：HTTP
- `ReadFile` / `ReadFileStream`：HTTP
- `ReadVersion`：读 data 时 HTTP，不读 data 时 grid
- `RenameData`：grid
- `WriteAll`：grid

更准确地说，MinIO 远程盘通信层本身就是混合协议：

1. HTTP 适合直接搬运流
2. grid 适合结构化 RPC 和流式结果

## 13. 远程盘不是“更弱的盘”，只是“跨节点访问”

从 `StorageAPI` 设计和远程链路可以看出：

1. 上层照样可以对远程盘调用 `CreateFile`
2. 上层照样可以对远程盘调用 `RenameData`
3. 上层照样把远程盘纳入 quorum

也就是说，远程盘在对象层语义上不是次等盘，而是等价盘。

差别只在：

1. 本地盘是直接函数调用
2. 远程盘是网络调用再落到远端本地盘

## 14. 远程盘与本地盘在 `erasureSets` 中如何被接入

文件：`minio/cmd/erasure-sets.go`

关键点：

1. `connectEndpoint()` 调 `newStorageAPI(endpoint, ...)`
2. 因此每个 endpoint 最终都会被包装成 `StorageAPI`
3. 后续 `erasureSets` / `erasureObjects` 只面对 `StorageAPI`

这意味着对象层并不为远程盘单独写一套逻辑，它只消费统一的 `StorageAPI` 列表。

所以 MinIO 的真正设计重点是：

1. 把本地盘和远程盘都压平到 `StorageAPI`
2. 保证二者在错误语义、disk-id、提交语义上足够一致

## 15. PutObject 视角下的最终链路

把远程盘代入 `PutObject` 主链后，可以得到：

### 15.1 写临时 shard

`PutObjectHandler`
-> `ObjectLayer.PutObject`
-> `erasureObjects.putObject`
-> `Erasure.Encode`
-> `newStreamingBitrotWriter`
-> `StorageAPI.CreateFile`

此时分两种情况：

1. 本地盘：
   `xlStorageDiskIDCheck.CreateFile -> xlStorage.CreateFile -> writeAllDirect`
2. 远程盘：
   `storageRESTClient.CreateFile -> storageRESTServer.CreateFileHandler -> 远端 xlStorageDiskIDCheck.CreateFile -> 远端 xlStorage.CreateFile -> writeAllDirect`

### 15.2 提交正式对象

`renameData`
-> `StorageAPI.RenameData`

此时分两种情况：

1. 本地盘：
   `xlStorageDiskIDCheck.RenameData -> xlStorage.RenameData`
2. 远程盘：
   `storageRESTClient.RenameData -> storageRESTServer.RenameDataHandler -> 远端 xlStorageDiskIDCheck.RenameData -> 远端 xlStorage.RenameData`

所以远程盘并没有改变 MinIO PutObject 的两阶段存储模型。

## 16. 是否还有“第三种盘”

从这套实现看，MinIO 在 `StorageAPI` 这一层的主要盘类型就是：

1. 本地盘
2. 远程盘

此外还有一些运行态状态：

1. 在线盘
2. 离线盘
3. stale disk
4. faulty disk
5. unformatted disk

但这些是“状态”，不是第三种独立存储实现。

所以如果只问存储访问实现类型，答案仍然是：

1. 本地 `xlStorage`
2. 远程 `storageRESTClient -> storageRESTServer -> 远端 xlStorage`

## 17. 对 fs3 的直接启示

如果 fs3 要对齐 MinIO 的“远程盘模型”，重点不在于发明另一套存储格式，而在于：

1. 先有统一的 `StorageAPI`
2. 本地实现和远程实现都满足同一接口
3. 远程实现的最终语义必须仍收敛到远端节点本地存储实现
4. 远程层要处理鉴权、disk-id、一致错误映射、健康检查
5. `CreateFile` 和 `RenameData` 的远程路径不能改变本地提交语义

换句话说，MinIO 的远程盘设计重点是“远程复用本地盘语义”，而不是“为远程盘重新设计一套存储层”。

## 18. 结论

MinIO 对盘访问路径的真实抽象可以总结为：

1. 本地盘：`xlStorageDiskIDCheck -> xlStorage`
2. 远程盘：`storageRESTClient -> storageRESTServer -> 远端 xlStorageDiskIDCheck -> 远端 xlStorage`

因此：

1. MinIO 确实区分本地盘和远程盘
2. 但远程盘不是第三种存储格式
3. 远程盘只是通过 internode 协议访问另一台机器上的本地盘
4. 最终真正的存储语义仍由 `xlStorage` 决定
5. `PutObject` 的两阶段模型在远程盘上并没有改变，只是多了一层跨节点调用

## 19. fs3 应如何抽象 remote storage 才接近 MinIO

下面不讨论“先做一个能跑的分布式 demo”，而讨论“fs3 如果要在抽象层接近 MinIO，需要怎样设计 remote storage”。

核心原则只有一句话：

fs3 不应该把 remote storage 设计成另一套存储后端，而应该设计成“远程访问另一节点本地存储实现”的统一 `StorageAPI` 实现。

## 20. 先明确目标，不要走偏

如果目标是接近 MinIO，fs3 的 remote storage 设计目标应该是：

1. 对上层只暴露统一 `StorageAPI`
2. 本地存储和远程存储共享同一组存储语义
3. `PutObject` 的 `tmp -> rename_data -> cleanup` 提交模型在远程路径上不变
4. 远程层只负责“跨节点转发 + 错误折叠 + 身份校验 + 健康检查”
5. 真正的存储语义仍由远端节点本地存储实现决定

不应该做成：

1. 远程节点直接操作本地文件路径字符串
2. 单独写一套 “remote_xl_storage”
3. 让远程路径和本地路径使用不同的提交顺序
4. 在对象层到处写 `if local else remote`

## 21. fs3 需要的总体分层

fs3 如果要贴近 MinIO，推荐分成四层：

1. `StorageAPI<E>`
2. `LocalStorage`
3. `RemoteStorageClient`
4. `RemoteStorageServer`

对应关系：

1. `LocalStorage`
   当前就是 `XlStorage` 或其后续包装层
2. `RemoteStorageClient`
   对外实现 `StorageAPI<E>`，但内部通过网络请求远端节点
3. `RemoteStorageServer`
   暴露节点内网接口，把请求转发给本机 `LocalStorage`
4. 上层 `ErasureServerPools` / object layer
   只依赖 `StorageAPI<E>`

这和 MinIO 的形状是一致的：

1. 本地盘直接实现真实存储语义
2. 远程盘复用本地盘语义

## 22. fs3 应增加的角色划分

### 22.1 本地真实存储实现

建议明确一个角色：

- `XlStorage` 负责真实存储语义

它应负责：

1. `create_file`
2. `write_all`
3. `rename_data`
4. `read_version`
5. `delete_path`

以及：

1. data dir 路径规则
2. `xl.meta` 编解码
3. `rename_data` 提交顺序
4. old data dir 清理返回值

### 22.2 本地包装层

建议新增一层本地包装，例如：

- `XlStorageChecked`

职责类似 MinIO 的 `xlStorageDiskIDCheck`：

1. 节点内 disk-id / storage-id 校验
2. timeout / cancel 包装
3. metrics
4. health 状态
5. 错误归一化

这一层不是必须第一天做完，但架构上应该预留。

### 22.3 远程客户端

建议新增：

- `RemoteStorageClient`

它应：

1. 实现 `StorageAPI<FS3Error>`
2. 内部持有远端节点地址、目标 storage id、连接状态
3. 对上层表现成“像本地盘一样可调用”

它不应：

1. 直接实现真正文件系统语义
2. 自己决定 `rename_data` 的提交顺序
3. 直接拼接远端磁盘文件路径做本地假设

### 22.4 远程服务端

建议新增：

- `RemoteStorageServer`

它应：

1. 暴露 internode API
2. 根据请求中的 endpoint/storage id 找到本机真实 `StorageAPI`
3. 做鉴权、时钟校验、storage-id 校验
4. 再调用本机 `XlStorage` 或包装层

这和 MinIO 的 `storageRESTServer` 角色一致。

## 23. fs3 的关键原则：远程实现也要实现 `StorageAPI`

这是最重要的一条。

fs3 不应该让上层对象层知道“这个盘是本地还是远程”。

正确做法是：

1. `XlStorage` 实现 `StorageAPI<FS3Error>`
2. `RemoteStorageClient` 也实现 `StorageAPI<FS3Error>`
3. `ErasureServerPools` 持有 `Vec<Arc<dyn StorageAPI<FS3Error>>>`

这样上层代码才能保持统一：

1. `create_file`
2. `rename_data`
3. `read_version`
4. `delete_path`

全部都走同一接口。

如果对象层需要写成：

- `if local_storage { ... } else if remote_storage { ... }`

那说明抽象已经偏离 MinIO 了。

## 24. fs3 的 endpoint / storage identity 设计建议

MinIO 里真正重要的不只是 host，而是：

1. endpoint
2. disk-id

fs3 也应该区分：

1. `NodeId`
2. `StorageId`
3. `Endpoint`

建议至少有这些结构：

```rust
pub struct StorageEndpoint {
    pub node_id: String,
    pub address: String,
    pub storage_id: String,
    pub is_local: bool,
}
```

其中：

1. `address` 表示节点通信地址
2. `storage_id` 表示具体存储实例身份
3. `is_local` 用于初始化时决定走本地实现还是远程客户端

重点是：

fs3 不应把“远程盘身份”只绑定到 URL；还必须绑定到稳定的 `storage_id`。

否则后续节点重启、磁盘替换、挂载切换时，无法判断“还是不是那块盘”。

## 25. fs3 的 `new_storage_api()` 应该长什么样

建议未来有一个统一工厂函数，例如：

```rust
pub fn new_storage_api(
    endpoint: &StorageEndpoint,
    opts: StorageOpts,
) -> Result<Arc<dyn StorageAPI<FS3Error>>, FS3Error>
```

分流规则：

1. `endpoint.is_local == true`
   返回本地 `XlStorage` 或其包装层
2. `endpoint.is_local == false`
   返回 `RemoteStorageClient`

这一步就是 fs3 版的 MinIO `newStorageAPI()`。

## 26. fs3 远程协议层应该如何选型

MinIO 是混合的：

1. HTTP 传流
2. grid RPC 传结构化请求

fs3 不一定要照搬协议，但应该保留同样的“语义分层”。

建议：

1. 流式读写操作：
   `create_file`、`read_file_stream`
   用 HTTP streaming 或 gRPC streaming
2. 结构化元数据操作：
   `rename_data`、`write_all`、`read_version`
   用普通 RPC / JSON / msgpack / protobuf

不建议把所有操作都塞进一种非常粗糙的接口里，例如：

1. 所有操作都走一个 `/storage/op`
2. body 里再塞一个大 JSON 做多态分发

这样会让：

1. 流式传输不好处理
2. 错误语义混乱
3. 超时与 keepalive 不清晰

## 27. fs3 远程协议需要区分两类调用

### 27.1 流式数据调用

这类调用的数据量大、耗时长：

1. `create_file`
2. `read_file_stream`
3. 未来可能的 shard stream write

需要：

1. 真流式 body
2. 明确的超时与 keepalive
3. 请求未完成时连接保持
4. 远端错误能在流结束时传回

### 27.2 结构化提交调用

这类调用更适合 RPC：

1. `rename_data`
2. `write_all`
3. `read_version`
4. `delete_path`

需要：

1. 明确结构体参数
2. 明确结构体返回值
3. 稳定的错误映射

## 28. fs3 的 `rename_data` 远程语义必须保持不变

这是整个设计里最重要的要求之一。

无论本地还是远程，`rename_data` 都必须仍然表示：

1. 输入临时源路径
2. 输入新版本 `FileInfo`
3. 读取目标旧 `xl.meta`
4. 先写临时 `xl.meta`
5. 再 rename data dir
6. 最后 rename `xl.meta`
7. 返回 `old_data_dir` / cleanup 信息

也就是说：

1. `RemoteStorageClient::rename_data()` 不应自己实现提交逻辑
2. 它只应把请求发给远端
3. 真正事务仍由远端本地 `XlStorage::rename_data()` 决定

如果把提交语义拆成：

1. 客户端先发一个“写 meta”
2. 再发一个“rename data”
3. 再发一个“rename meta”

那就已经偏离 MinIO 了，因为这样远程层把提交事务拆散了。

正确做法是：

- `rename_data` 作为一个完整远程存储接口暴露出去

## 29. fs3 的 `create_file` 远程语义也必须保持不变

当前 fs3 `create_file()` 仍比较简化，但无论未来是否引入 shard/bitrot，都应坚持一条：

`RemoteStorageClient::create_file()` 只是把流转发给远端本地实现，而不是重新解释写入语义。

这意味着：

1. 现在的单文件流写入可远程转发
2. 将来如果变成 shard writer，也仍可远程转发
3. 将来如果 `create_file` 增加 fsync/direct-io/bitrot 选项，也应由远端本地实现负责

## 30. fs3 应增加的请求/响应类型

为了让远程层不污染对象层，建议单独新增 remote storage 协议类型，例如放到：

- `src/types/storage_remote/`

至少包含：

1. `CreateFileRequest`
2. `CreateFileResponse`
3. `RenameDataRequest`
4. `RenameDataResponse`
5. `WriteAllRequest`
6. `ReadVersionRequest`
7. `DeletePathRequest`
8. `StorageRemoteError`

其中 `RenameDataRequest` 应尽量直接承载：

1. `src_volume`
2. `src_path`
3. `fi`
4. `dst_volume`
5. `dst_path`
5. `opts`

这样远程层只是搬运 `StorageAPI` 语义，而不是重新发明语义。

## 31. fs3 的错误模型该怎么做

既然当前仓库要求“统一错误为 `FS3Error`”，那 remote storage 更应该做错误折叠。

建议：

1. 远程协议错误
2. 网络超时错误
3. 对端鉴权错误
4. storage-id mismatch
5. 对端返回的本地存储错误

最终都在客户端折叠成 `FS3Error`。

应明确增加几类错误分支，例如：

1. `FS3Error::StorageOffline`
2. `FS3Error::StorageStale`
3. `FS3Error::StorageUnauthorized`
4. `FS3Error::StorageTransport`
5. `FS3Error::StorageRemote`

这样对象层拿到的仍然是统一错误，而不是 HTTP status code 或 transport error。

## 32. fs3 必须有 storage-id 校验

这是 MinIO 设计里非常值得学的一点。

fs3 的远程服务端应在请求里校验：

1. 请求目标 storage id
2. 本机当前 storage id

如果不一致，应直接返回：

- stale / mismatch 错误

原因：

1. 节点重建后路径可能还是那个路径
2. 但底层磁盘可能已经不是原来的实例
3. 如果没有 storage-id 校验，会把“盘替换”误判成“同一块盘恢复上线”

## 33. fs3 必须有 internode 鉴权

如果以后有 remote storage，就不能把 storage API 暴露成裸接口。

至少应有：

1. 节点间共享密钥或签名 token
2. 请求时间校验
3. 可选的 node-id 校验

否则：

1. 内网任意人都能调用底层存储接口
2. 很容易破坏对象层事务语义

这部分不一定要完全照搬 MinIO 的 JWT，但语义上必须等价。

## 34. fs3 应有健康检查和连接状态

MinIO 远程客户端不只是“发请求失败就算了”，而是有：

1. 在线状态
2. 上次连接时间
3. health check
4. 网络错误到磁盘错误的映射

fs3 也应有类似能力。

建议 `RemoteStorageClient` 至少维护：

1. `is_online`
2. `last_conn`
3. `storage_id`
4. `endpoint`

并提供：

1. 定期 health probe
2. 失败重试策略
3. 错误映射到 `FS3Error`

## 35. fs3 对象层不应直接依赖 remote 细节

对象层应继续只做这些事：

1. 选盘
2. 写临时对象
3. 调 `rename_data`
4. 删除 old data dir
5. 做 quorum 聚合

它不应关心：

1. HTTP 路径
2. RPC 编码
3. 连接池
4. token 校验

这些都应被 remote storage 层吃掉。

## 36. fs3 如果将来做多盘，remote storage 才真正有价值

当前 fs3 还是单机简化模型，所以 remote storage 不是第一优先级实现项。

但如果目标真是接近 MinIO，那么 remote storage 的价值在于：

1. 多节点时把远端盘纳入统一 `StorageAPI`
2. 为 quorum 提供真实远端盘
3. 让 shard 写入可以分发到不同节点

所以 remote storage 最合理的落地顺序应是：

1. 先把本地 `StorageAPI` 语义稳定
2. 再做一层 `RemoteStorageClient/Server`
3. 再让 `ErasureServerPools` 接受本地和远程混合盘

## 37. 建议的最小落地顺序

### 37.1 第一阶段

先定义抽象，不做真正分布式：

1. 增加 `StorageEndpoint`
2. 增加 `new_storage_api()`
3. 增加 `RemoteStorageClient` / `RemoteStorageServer` 类型骨架
4. 保证 `RemoteStorageClient` 实现 `StorageAPI<FS3Error>`

### 37.2 第二阶段

先实现最小闭环：

1. `read_version`
2. `write_all`
3. `rename_data`
4. `delete_path`

这些都是结构化调用，最容易先打通。

### 37.3 第三阶段

再实现流式大数据调用：

1. `create_file`
2. `read_file_stream`

这样就能让 `PutObject` 主链真正跨节点工作。

### 37.4 第四阶段

最后再补：

1. storage-id 校验
2. health check
3. metrics
4. 重试与错误折叠
5. 未来 shard / bitrot / quorum 的远程协同

## 38. 一个更接近 MinIO 的最小接口示意

下面这个方向比现在直接把远程逻辑塞进 object layer 更接近 MinIO：

```rust
#[async_trait]
pub trait StorageAPI<E>: Send + Sync
where
    E: StdError,
{
    fn endpoint(&self) -> &StorageEndpoint;
    fn is_local(&self) -> bool;
    fn storage_id(&self) -> Option<&str>;

    async fn read_version(&self, ctx: &Context, volume: &str, path: &str, version_id: &str) -> Result<FileInfo, E>;
    async fn write_all(&self, ctx: &Context, volume: &str, path: &str, data: &[u8], opts: WriteAllOptions) -> Result<(), E>;
    async fn rename_data(&self, ctx: &Context, src_volume: &str, src_path: &str, fi: FileInfo, dst_volume: &str, dst_path: &str, opts: RenameDataOptions) -> Result<RenameDataResult, E>;
    async fn create_file(&self, ctx: &Context, volume: &str, path: &str, size: i64, reader: BoxByteStream, opts: CreateFileOptions) -> Result<u64, E>;
    async fn delete_path(&self, ctx: &Context, volume: &str, path: &str, opts: DeletePathOptions) -> Result<(), E>;
}
```

重点不是字段名，而是：

1. 本地与远程必须共享同一批关键语义接口
2. `rename_data` 必须保留为单个完整事务调用
3. `create_file` 必须保留为流式写入接口

## 39. 最终设计判断

如果 fs3 以后要接近 MinIO，最正确的 remote storage 设计不是：

- “给远程盘单独做一套对象写入逻辑”

而是：

- “让远程盘成为另一个 `StorageAPI` 实现，并把真正存储语义继续留在远端本地存储实现里”

因此，fs3 应该把 remote storage 设计成：

1. `new_storage_api()` 统一分流
2. `RemoteStorageClient` 实现 `StorageAPI<FS3Error>`
3. `RemoteStorageServer` 把请求转发给远端本地 `XlStorage`
4. 通过 storage-id、鉴权、健康检查、错误折叠来保证它像 MinIO 那样可被当成“远程盘”

## 40. 这一节的结论

fs3 若想接近 MinIO，remote storage 的正确方向是：

1. 不新增另一套存储格式
2. 不让对象层知道本地/远程细节
3. 保持 `StorageAPI` 统一
4. 让远程层只做“网络化的存储调用适配”
5. 让真实存储语义仍由远端本地 `XlStorage` 决定

换句话说，MinIO 的 remote storage 设计本质上不是“remote-first”，而是“local-storage semantics over remote transport”。
