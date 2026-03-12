# fs3 PutObject 与 MinIO 架构差异

本文档对比当前 `src/` 下 fs3 的 PutObject 请求链路与 MinIO 的真实 PutObject 调用链，重点记录结构性差异，而不是零散实现细节。

参考基线：

- MinIO 调用链文档：`docs/minio_actual/put_object_call_stack.md`
- fs3 当前代码：`src/`

## 1. 当前 fs3 的 PutObject 实际调用链

当前 fs3 普通 PutObject 的实际路径是：

1. `axum` 路由进入 object handler
2. `S3Handler` trait 默认实现处理请求
3. `FS3Engine::put_object`
4. `ObjectLayer::put_object`
5. `ErasureServerPools::put_object`
6. `XlStorage::create_file`
7. `XlStorage::write_metadata`

对应源码：

- 路由入口：[src/components/fs3_axum_handler/http_object.rs](/D:/1-code/__trash__/fs3/src/components/fs3_axum_handler/http_object.rs:42)
- 普通 PutObject 分支：[src/components/fs3_axum_handler/http_object.rs](/D:/1-code/__trash__/fs3/src/components/fs3_axum_handler/http_object.rs:188)
- Handler trait 默认实现：[src/types/traits/s3_handler/object.rs](/D:/1-code/__trash__/fs3/src/types/traits/s3_handler/object.rs:405)
- Engine：[src/components/fs3_engine/object.rs](/D:/1-code/__trash__/fs3/src/components/fs3_engine/object.rs:10)
- ObjectLayer trait：[src/types/traits/object_layer.rs](/D:/1-code/__trash__/fs3/src/types/traits/object_layer.rs:16)
- ObjectLayer 实现：[src/components/erasure_server_pools/object.rs](/D:/1-code/__trash__/fs3/src/components/erasure_server_pools/object.rs:77)
- Storage 写文件：[src/components/xl_storage/mod.rs](/D:/1-code/__trash__/fs3/src/components/xl_storage/mod.rs:265)
- Storage 写元数据：[src/components/xl_storage/mod.rs](/D:/1-code/__trash__/fs3/src/components/xl_storage/mod.rs:155)

## 2. 与 MinIO 的核心结构差异

## 2.1 请求入口不是流式

fs3 在 HTTP 入口已经把整个对象 body 读进内存。

源码：

- [src/components/fs3_axum_handler/http_object.rs](/D:/1-code/__trash__/fs3/src/components/fs3_axum_handler/http_object.rs:42)
- [src/components/fs3_axum_handler/http_object.rs](/D:/1-code/__trash__/fs3/src/components/fs3_axum_handler/http_object.rs:48)
- [src/components/fs3_axum_handler/http_object.rs](/D:/1-code/__trash__/fs3/src/components/fs3_axum_handler/http_object.rs:201)

关键点：

- `object_entry(...)` 参数里直接是 `body: Bytes`
- 普通 PutObject 再把 `body` 包成 `stream::once(...)`

这与 MinIO 不同：

- MinIO 的 `PutObjectHandler` 直接从 `r.Body` 开始构造 reader
- 后续 checksum、签名流校验、SSE、压缩都建立在流式读取之上

影响：

1. 大对象会先整体占用内存
2. 不支持真正的流式 backpressure
3. chunked signed upload 行为无法对齐 MinIO
4. handler 层的 MD5/checksum 校验只能走“先读完再算”

## 2.2 handler 层仍保留原始 XML / 原始 body

当前请求模型仍然允许原始 XML 字符串和原始 body 在 handler 里流动。

源码：

- PutObject request：[src/types/s3/request.rs](/D:/1-code/__trash__/fs3/src/types/s3/request.rs:254)
- PutObject from axum：[src/types/s3/request_from_axum.rs](/D:/1-code/__trash__/fs3/src/types/s3/request_from_axum.rs:297)
- PutObjectTagging raw xml：[src/components/fs3_axum_handler/http_object.rs](/D:/1-code/__trash__/fs3/src/components/fs3_axum_handler/http_object.rs:162)
- PutObjectRetention raw xml：[src/components/fs3_axum_handler/http_object.rs](/D:/1-code/__trash__/fs3/src/components/fs3_axum_handler/http_object.rs:165)
- PutObjectLegalHold raw xml：[src/components/fs3_axum_handler/http_object.rs](/D:/1-code/__trash__/fs3/src/components/fs3_axum_handler/http_object.rs:168)

这与仓库设计准则冲突：

- XML 请求应在入口解码成结构体
- 裸 XML 不应进入 engine 层及以下

也与 MinIO 的行为不同：

- MinIO 在 handler 内部会把请求头、body、query 解成具体结构后再进入对象层

## 2.3 中间件语义远少于 MinIO

fs3 当前 PutObject 主要只做了：

1. 路由分发
2. policy access check
3. 可选的 `Content-MD5` 校验

源码：

- access check：[src/types/traits/s3_handler/object.rs](/D:/1-code/__trash__/fs3/src/types/traits/s3_handler/object.rs:406)
- MD5 校验：[src/types/traits/s3_handler/object.rs](/D:/1-code/__trash__/fs3/src/types/traits/s3_handler/object.rs:414)

而 MinIO 的 `PutObjectHandler` 还包含：

1. auth type 处理
2. date/skew 校验
3. content-length 校验
4. copy-source/header 冲突校验
5. storage-class 校验
6. quota 校验
7. object lock / retention / legal hold
8. replication
9. checksum
10. SSE/SSE-C/SSE-KMS
11. compression
12. precondition / conditional write

当前 fs3 的 handler/engine/object layer 结构本身还没有承载这些选项的类型能力。

## 2.4 Content-MD5 校验方式错误地依赖整包缓存

源码：

- [src/types/traits/s3_handler/object.rs](/D:/1-code/__trash__/fs3/src/types/traits/s3_handler/object.rs:415)

当前流程：

1. `req.body.try_collect()` 收集全部 chunk
2. 拼出完整 buffer
3. 计算 MD5
4. 再重新构造一次 `stream::once(...)`

这与 MinIO 的差异：

- MinIO 边读边校验 hash
- 不需要把整个对象读入内存

影响：

1. 大对象内存占用不合理
2. 与真实 S3/MinIO 流式校验行为不一致
3. 后续加 checksum / SSE / streaming signature 时会继续冲突

## 3. 对象层差异

## 3.1 ObjectOptions 过于简化

fs3 当前对象层选项：

- [src/types/s3/object_layer_types.rs](/D:/1-code/__trash__/fs3/src/types/s3/object_layer_types.rs:9)

```rust
pub struct ObjectOptions {
    pub version_id: Option<String>,
    pub user_defined: HashMap<String, String>,
    pub range: Option<(u64, u64)>,
}
```

这只能支持非常基础的 object read/write。

MinIO PutObject 路径实际需要的语义远更多，例如：

1. precondition callback
2. versioning flags
3. mtime
4. checksum 需求
5. checksum 结果
6. encryption function
7. index callback
8. retention / legal hold
9. replicate / replica 元数据
10. lock / no lock
11. overwrite / data movement 等提交语义

结论：

- 当前 `ObjectOptions` 无法承载 MinIO PutObject 的真实执行过程

## 3.2 ErasureServerPools 名字与实现不一致

源码：

- [src/components/erasure_server_pools/object.rs](/D:/1-code/__trash__/fs3/src/components/erasure_server_pools/object.rs:77)

当前 `put_object()` 实际做的事：

1. 生成 `version_id`
2. 生成 `data_dir`
3. 拼成 `object/data_dir`
4. 直接 `create_file(...)`
5. 直接 `write_metadata(...)`

核心代码：

```rust
81: let file_path = format!("{}/{}", object, data_dir);
82: let actual_size = self.storage.create_file(ctx, bucket, &file_path, data.size, data.reader).await?;
95: self.storage.write_metadata(ctx, bucket, object, fi).await?;
```

问题：

- 没有 pool 选择
- 没有多磁盘分布
- 没有 shard 切分
- 没有 bitrot
- 没有 write quorum
- 没有临时对象目录
- 没有 `RenameData` 提交

所以它现在不是 MinIO 意义上的 `erasureServerPools`，而是“单路径文件写入 + xl.meta”。

## 3.3 PutObject 事务模型与 MinIO 完全不同

MinIO：

1. 写临时对象
2. 生成临时 `xl.meta`
3. `RenameData()` 原子提交
4. 清理旧 data dir

当前 fs3：

1. 直接写正式 data path
2. 再直接写正式 `xl.meta`

源码：

- 正式 data 写入：[src/components/erasure_server_pools/object.rs](/D:/1-code/__trash__/fs3/src/components/erasure_server_pools/object.rs:81)
- 正式 metadata 写入：[src/components/erasure_server_pools/object.rs](/D:/1-code/__trash__/fs3/src/components/erasure_server_pools/object.rs:95)

这会导致以下差异：

1. 中途失败时磁盘状态和 MinIO 不一致
2. 覆盖写时无法模拟 MinIO 的 oldDataDir 清理语义
3. crash recovery 行为不同
4. 写入可见性窗口不同

## 4. 存储抽象差异

## 4.1 StorageAPI 缺少 RenameData / WriteAll 级别抽象

源码：

- [src/types/traits/storage_api.rs](/D:/1-code/__trash__/fs3/src/types/traits/storage_api.rs:23)

当前接口只有：

1. `create_file`
2. `append_file`
3. `rename_file`
4. metadata 读写

缺少 MinIO PutObject 提交阶段最关键的抽象：

1. `RenameData`
2. `WriteAll`
3. Delete old data dir with commit semantics

结果：

- 上层没法表达“先写 tmp，再提交 data dir + xl.meta”
- 只能做“创建文件 + 写 metadata”

## 4.2 XlStorage::create_file 只是单文件流写

源码：

- [src/components/xl_storage/mod.rs](/D:/1-code/__trash__/fs3/src/components/xl_storage/mod.rs:265)

```rust
265: async fn create_file(&self, _ctx: &Context, volume: &str, path: &str, _size: i64, mut reader: BoxByteStream) -> Result<u64, StorageError> {
271:     let mut file = tokio::fs::File::create(&file_path).await
276:     while let Some(chunk) = reader.next().await {
278:         file.write_all(&bytes).await
282:     Ok(total)
}
```

特点：

- 单文件写入
- 不做 bitrot
- 不做 shard
- 不做 fsync 语义控制
- 不做 direct I/O
- 不做 quorum

与 MinIO 的 `CreateFile -> writeAllDirect` 差异很大。

## 4.3 XlStorage::write_metadata 直接重写 xl.meta

源码：

- [src/components/xl_storage/mod.rs](/D:/1-code/__trash__/fs3/src/components/xl_storage/mod.rs:155)

当前做法：

1. 构造一个新的 `XlMetaV2`
2. 只写一个 shallow version
3. 直接 `tokio::fs::write(meta_path, data)`

问题：

- 没有读取并合并现有 `xl.meta`
- 没有版本事务
- 没有覆盖时保留旧版本
- 没有 oldDataDir
- 没有与 data dir rename 绑定提交

## 4.4 小对象 inline 策略与 MinIO 写入链不一致

源码：

- [src/components/xl_storage/mod.rs](/D:/1-code/__trash__/fs3/src/components/xl_storage/mod.rs:167)

当前逻辑：

1. 先把数据写成文件
2. `write_metadata()` 时如果小于 128 KiB，再回读这个文件
3. 放入 `inline_data`
4. 删除原文件

这不是 MinIO PutObject 的提交流程。

影响：

1. 多了一次“写完再读回”的窗口
2. 失败恢复行为不同
3. 和 MinIO inline data 的生成时机不同

## 5. 元数据与读取路径差异

## 5.1 read_version 没有按 version_id 查找

源码：

- [src/components/xl_storage/mod.rs](/D:/1-code/__trash__/fs3/src/components/xl_storage/mod.rs:125)

当前虽然传入 `version_id`，但实际逻辑是：

```rust
135: let version = &xl_meta.versions[0];
```

这说明当前版本读取还只是“存了 version_id 字段”，没有真正实现版本寻址。

结果：

1. 覆盖写行为无法对齐 MinIO
2. 指定版本读取无法对齐 MinIO
3. 删除版本行为无法对齐 MinIO

## 5.2 get_object_info / get_object 仍有明显占位实现

源码：

- [src/components/erasure_server_pools/object.rs](/D:/1-code/__trash__/fs3/src/components/erasure_server_pools/object.rs:11)
- [src/components/erasure_server_pools/object.rs](/D:/1-code/__trash__/fs3/src/components/erasure_server_pools/object.rs:27)

明显问题：

1. `get_object_info()` 返回 `etag: ""`
2. `get_object()` 中 `ObjectInfo.bucket` 被错误地赋成 `ctx.request_id`
3. `content_type` 大多是硬编码 `"application/octet-stream"`
4. `etag` 与 MinIO 语义没有对齐

这些都说明 object layer 还没有形成稳定的 MinIO 兼容语义。

## 6. 当前 fs3 与 MinIO 的本质差距

当前 fs3 更接近：

- 有 S3 风格 API 外壳
- 有分层架构
- 有 MinIO `xl.meta` 编解码能力
- 有单机本地文件落盘

但距离 MinIO PutObject 的真实模型还差以下关键部分：

1. 流式请求入口
2. 真实 handler 中间语义
3. 丰富的 object layer write options
4. erasure encode
5. shard 写入
6. bitrot
7. quorum
8. tmp object 写入
9. `RenameData` 提交
10. oldDataDir 清理
11. 版本合并写入
12. 崩溃恢复一致性

## 7. 优先级最高的缺口

如果目标是向 MinIO PutObject 存储兼容靠近，优先级建议如下：

### 7.1 第一优先级

1. 将 `http_object` 改为流式请求体，不再使用 `Bytes` 全量缓冲
2. 清理 request/handler 中的裸 XML / 裸 body 传播
3. 让 `PutObjectRequest` 只承载已解析字段与流式 body

### 7.2 第二优先级

1. 扩展 `ObjectOptions`
2. 扩展 `StorageAPI`
3. 增加 `write_all`
4. 增加 `rename_data`
5. 增加提交阶段所需返回值与 oldDataDir 语义

### 7.3 第三优先级

1. 重写 `ErasureServerPools::put_object`
2. 改成 `tmp write -> metadata build -> rename commit`
3. 不再“直接写正式 data + 直接写正式 xl.meta”

### 7.4 第四优先级

1. 真正实现版本读取/写入合并
2. 对齐 null version / overwrite / old data dir
3. 修正 etag / content-type / object info 生成逻辑

## 8. 结论

当前 fs3 的 PutObject 链路已经有合理的分层骨架：

`axum -> s3_handler -> s3_engine -> object_layer -> storage`

但它与 MinIO 的差异不是局部函数行为问题，而是写入事务模型不同。

当前 fs3 的核心问题可以概括为三条：

1. 请求入口不是流式
2. 对象层没有 `tmp -> RenameData commit` 的事务模型
3. 存储层只是单文件写入，并不具备 MinIO 的 erasure/shard/bitrot/quorum 语义

这三条如果不先修，后续补 checksum、SSE、retention、replication 等功能都会继续建立在错误的 PutObject 主链之上。
