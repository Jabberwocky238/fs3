# fs3 PutObject 与 MinIO 架构差异

本文档对比当前 `src/` 下 fs3 的 PutObject 请求链路与 MinIO 的真实 PutObject 调用链，重点记录截至当前代码状态仍然存在的结构性差异，并删除已经不属实的旧结论。

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
7. `XlStorage::rename_data`

对应源码：

- 路由入口：`src/components/fs3_axum_handler/http_object.rs`
- Handler trait 默认实现：`src/types/traits/s3_handler/object.rs`
- Engine：`src/components/fs3_engine/object.rs`
- ObjectLayer trait：`src/types/traits/object_layer.rs`
- ObjectLayer 实现：`src/components/erasure_server_pools/object.rs`
- Storage 写文件：`src/components/xl_storage/mod.rs`
- Storage 提交：`src/components/xl_storage/mod.rs`

## 2. 已经修正、旧文档不再成立的部分

## 2.1 请求入口已经改为流式

旧文档中“fs3 在 HTTP 入口已经把整个对象 body 读进内存”的结论，当前已不成立。

当前 `axum` 路由层已经把 `Body` 转成 `BoxByteStream`，普通 PutObject 直接把流式 body 传入 `PutObjectRequest`：

- `body.into_data_stream()`
- `PutObjectRequest { body: body_stream(body), ... }`

这说明 PutObject 请求入口已经从“整包 `Bytes` 缓冲”切换到了“流式 body 传递”。

## 2.2 对象相关 XML 已在入口解码

旧文档中“handler 层仍保留原始 XML / 原始 body”这一结论，对对象相关 XML 请求已不成立。

当前对象路由里：

1. 先通过 `body_text(body)` 读取文本
2. 再调用 `xml::parse_*`
3. 将解析后的结构体或字段传给 handler

已确认修正的对象相关分支包括：

- `PutObjectAcl`
- `PutObjectTagging`
- `PutObjectRetention`
- `PutObjectLegalHold`
- `CompleteMultipartUpload`
- `SelectObjectContent`
- `PostRestoreObject`

因此，这些对象 API 已不再把裸 XML 直接传到 engine 层。

## 2.3 StorageAPI 已补上提交阶段抽象

旧文档中“StorageAPI 缺少 `RenameData` / `WriteAll` 级别抽象”的结论已不成立。

当前 `StorageMetadata` trait 已包含：

1. `write_all`
2. `write_metadata`
3. `rename_data`

这说明上层已经可以表达“先写临时对象，再提交 metadata/data”的提交阶段语义。

## 2.4 PutObject 已切换到临时目录再提交

旧文档中“当前 fs3 直接写正式 data path，再直接写正式 xl.meta”的结论，对普通 PutObject 已不成立。

当前 `ErasureServerPools::put_object()` 的流程是：

1. 生成 `version_id`
2. 生成 `data_dir`
3. 生成临时对象 ID
4. 写入 `.minio.sys/tmp/{temp_object}/{data_dir}`
5. 调用 `storage.rename_data(...)` 提交到正式 `bucket/object`

因此，普通 PutObject 已经具备了基本的 `tmp -> commit` 写入模型。

## 2.5 版本读取不再固定取第一个版本

旧文档中“`read_version` 没有按 `version_id` 查找”的结论已不成立。

当前 `XlStorage::read_version()` 已实现：

- `version_id == "null"` 时取首版本
- 否则按 `version.header.version_id` 精确匹配

这说明版本读取已经从占位实现前进到真实按版本查找。

## 3. 当前仍然存在的核心结构差异

## 3.1 流式入口已修复，但流式校验语义仍未补齐

虽然 PutObject 入口已经变成流式 body，但 MinIO `PutObjectHandler` 上层的大量流式语义当前仍未实现，例如：

1. 边读边校验 `Content-MD5`
2. 流式 checksum
3. chunked signed upload
4. SSE / SSE-C / SSE-KMS
5. 压缩、加密和 hash 叠加 reader

当前 handler 中 `content_md5` 只是读取出来后丢弃：

- `let _content_md5 = req.content_md5;`

因此，“入口流式化”已经完成，但“基于流的真实校验链”仍未建立。

## 3.2 中间件语义仍远少于 MinIO

fs3 当前 PutObject 主路径主要仍只做了：

1. 路由分发
2. policy access check
3. 基础 header 提取

而 MinIO 的 `PutObjectHandler` 还包含大量当前缺失的处理：

1. auth type 处理
2. date/skew 校验
3. 更完整的 content-length 约束
4. storage-class 校验
5. quota 校验
6. object lock / retention / legal hold 权限联动
7. checksum 语义
8. SSE/SSE-C/SSE-KMS
9. replication
10. precondition / conditional write

当前 fs3 的 handler / engine / object layer 类型能力仍不足以承载这些语义。

## 3.3 ObjectOptions 仍然过于简化

当前对象层选项仍只有：

1. `version_id`
2. `user_defined`
3. `range`

这只能支撑非常基础的 object read/write。

相对 MinIO PutObject 路径，当前仍缺少承载以下语义的字段能力：

1. checksum 需求和结果
2. precondition callback
3. overwrite / commit 相关选项
4. retention / legal hold
5. encryption 参数
6. lock / no lock
7. index callback
8. replicate / replica 元数据

## 3.4 ErasureServerPools 只修了提交模型，没有实现真正的 erasure 语义

当前 `ErasureServerPools::put_object()` 的确已经切到：

- 临时写入
- `rename_data` 提交

但它仍不是 MinIO 意义上的 `erasureServerPools`，因为它还没有：

1. pool 选择
2. 多磁盘分布
3. shard 切分
4. bitrot writer
5. write quorum
6. 纠删码编码

当前本质上仍是：

- 单文件流写入
- 单机本地目录提交
- 再生成 `xl.meta`

只是事务模型比旧版本更接近 MinIO。

## 3.5 存储层只实现了“单文件 + rename 提交”，没有 MinIO 最终写入语义

`XlStorage::create_file()` 当前仍然只是：

1. 创建父目录
2. 创建文件
3. 顺序写入 stream

它仍然没有 MinIO `CreateFile -> writeAllDirect` 里的这些特征：

1. shard 级写入
2. bitrot hash 写入
3. `fdatasync` / 落盘语义
4. direct I/O 语义
5. quorum 失败处理

因此，fs3 目前只是“借用了 MinIO 类似的提交阶段形状”，还没有 MinIO 的底层落盘语义。

## 3.6 `rename_data()` 只实现了简化版事务，不是完整 MinIO 事务

当前 `XlStorage::rename_data()` 已经做了这些事情：

1. 读取目标现有 `xl.meta`
2. 将新版本插入到最前
3. 把新 `xl.meta` 先写到临时目录
4. rename data dir
5. rename xl.meta
6. 删除旧 data dir
7. 清理临时目录

这已经非常接近 MinIO PutObject 的核心提交顺序。

但与 MinIO 真实 `RenameData()` 相比，当前仍然是简化版，仍缺少：

1. 多磁盘/多返回值聚合
2. quorum 语义
3. 更完整的版本事务处理
4. 更完整的恢复与失败回滚语义

## 3.7 元数据写入与读取仍有明显占位问题

虽然版本查找已经修正，但对象读路径仍存在明显占位实现：

1. `get_object_info()` 返回 `etag: ""`
2. `get_object()` 把 `ObjectInfo.bucket` 错填为 `ctx.request_id`
3. `content_type` 仍大量硬编码为 `"application/octet-stream"`
4. `etag` 仍未对齐 MinIO 语义

这说明 object layer 还没有形成稳定的 MinIO 兼容读写元数据语义。

## 3.8 CopyObject 仍未切换到新事务模型

普通 PutObject 现在已经是：

- `tmp write -> rename_data`

但当前 `copy_object()` 仍然是：

1. 直接写目标 data path
2. 调用 `write_metadata()`

因此 CopyObject 仍停留在旧事务模型，尚未与 PutObject 的新主链统一。

## 3.9 小对象 inline 逻辑仍未对齐 MinIO

当前代码仍然保留 `inline_data` 方向的实现痕迹，但这部分与当前提交模型尚未形成稳定闭环。

就算后续编译修复完成，它与 MinIO 的真实写入链也仍有待核对：

1. inline data 的生成时机
2. 与普通 data dir 提交的关系
3. 覆盖写和版本合并时的行为

这部分不应视为已完成兼容。

## 4. 当前 fs3 与 MinIO 的本质差距

当前 fs3 已经从原来的：

- “单文件直接写正式路径”

前进到了：

- “流式入口 + 临时写入 + rename 提交”

这是重要进展。

但距离 MinIO PutObject 的真实模型仍然还有以下关键缺口：

1. 完整的流式校验链
2. 丰富的 handler 中间语义
3. 更完整的 object layer write options
4. erasure encode
5. shard 写入
6. bitrot
7. quorum
8. 完整版本事务
9. 读路径元数据语义对齐
10. 崩溃恢复一致性

## 5. 优先级最高的剩余缺口

如果目标是继续向 MinIO PutObject 存储兼容靠近，当前优先级建议如下：

## 5.1 第一优先级

1. 让当前“流式入口 + rename 提交”分支先编译通过并稳定下来
2. 补齐 `Content-MD5` / checksum 的流式校验
3. 修正当前 object info / etag / content-type 的占位实现

## 5.2 第二优先级

1. 扩展 `ObjectOptions`
2. 为 commit / overwrite / retention / checksum / encryption 增加类型能力
3. 统一 PutObject 与 CopyObject 的提交模型

## 5.3 第三优先级

1. 引入真正的 erasure encode
2. 改为 shard 写入
3. 引入 bitrot
4. 引入 quorum 语义

## 5.4 第四优先级

1. 对齐 null version / overwrite / old data dir 更多细节
2. 补齐崩溃恢复一致性
3. 对齐更多 handler 级 MinIO 行为

## 6. 结论

当前 fs3 的 PutObject 主链已经出现了实质性修正：

1. 请求入口已流式化
2. 对象相关 XML 已在入口解析
3. PutObject 已切换到 `tmp -> rename_data commit`
4. `read_version` 已支持按 `version_id` 查找

因此，旧文档里以下结论已经不再属实：

1. “PutObject 入口仍是整包内存缓冲”
2. “StorageAPI 缺少 `rename_data` / `write_all`”
3. “PutObject 仍直接写正式 data path 和正式 xl.meta”
4. “`read_version` 仍固定读取第一个版本”

但当前 fs3 与 MinIO 的差异依然不是局部函数行为问题，而是“已修正事务骨架，但底层写入语义和 handler 语义仍不完整”。

当前最准确的总结应当是：

1. PutObject 主链骨架已明显向 MinIO 靠拢
2. 提交模型已经从“直接写正式路径”升级为“临时写入后提交”
3. 但 erasure/shard/bitrot/quorum/checksum/SSE 等关键能力仍未完成
