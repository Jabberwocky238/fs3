# MinIO PutObject 请求调用栈

阅读D:\1-code\__trash__\fs3\docs\minio_actual\put_object_call_stack.md和D:\1-code\__trash__\fs3\docs\fs3_lackof\put_object_gap_vs_minio.md，使用UTF8

本文档记录 MinIO 普通对象上传请求 `PUT /{bucket}/{object}` 的实际调用路径，目标是从 HTTP 请求入口一路追到最终存储层写入。

范围说明：

- 只分析普通 `PutObject`
- 不包含 `CopyObject`
- 不包含 `PutObjectPart`
- 不包含 `PutObjectTagging`
- 不包含 `PutObjectRetention`
- 不包含 `PutObjectLegalHold`
- 不包含 `PutObjectExtract`

这些分支虽然也使用 `PUT /{bucket}/{object}`，但会在路由阶段因 query/header 条件优先匹配到其他 handler。

## 1. HTTP 服务入口

### 1.1 服务器装配入口

文件：[minio/cmd/server-main.go](/D:/1-code/__trash__/fs3/minio/cmd/server-main.go:893)

```go
893:        handler, err := configureServerHandler(globalEndpoints)
902:            UseHandler(setCriticalErrorHandler(corsHandler(handler))).
```

调用关系：

1. `configureServerHandler(globalEndpoints)`
2. `corsHandler(handler)`
3. `setCriticalErrorHandler(...)`
4. 交给 `xhttp.Server`

说明：

- `setCriticalErrorHandler()` 是最外层 panic/critical recover 包装。
- `corsHandler()` 在 S3 路由外层。

### 1.2 总路由构建

文件：[minio/cmd/routers.go](/D:/1-code/__trash__/fs3/minio/cmd/routers.go:84)

```go
84:func configureServerHandler(endpointServerPools EndpointServerPools) (http.Handler, error) {
87:    router := mux.NewRouter().SkipClean(true).UseEncodedPath()
95:    registerAdminRouter(router, true)
98:    registerHealthCheckRouter(router)
101:   registerMetricsRouter(router)
104:   registerSTSRouter(router)
107:   registerKMSRouter(router)
110:   registerAPIRouter(router)
112:   router.Use(globalMiddlewares...)
114:   return router, nil
}
```

说明：

- S3 API 路由由 `registerAPIRouter(router)` 注册。
- 全局中间件通过 `router.Use(globalMiddlewares...)` 统一应用。

## 2. 全局中间件

文件：[minio/cmd/routers.go](/D:/1-code/__trash__/fs3/minio/cmd/routers.go:53)

```go
53:var globalMiddlewares = []mux.MiddlewareFunc{
56:    addCustomHeadersMiddleware,
60:    httpTracerMiddleware,
66:    setAuthMiddleware,
69:    setBrowserRedirectMiddleware,
71:    setCrossDomainPolicyMiddleware,
73:    setRequestLimitMiddleware,
75:    setRequestValidityMiddleware,
77:    setUploadForwardingMiddleware,
79:    setBucketForwardingMiddleware,
}
```

对普通 `PutObject` 关键的是以下几层：

1. `addCustomHeadersMiddleware`
2. `httpTracerMiddleware`
3. `setAuthMiddleware`
4. `setRequestLimitMiddleware`
5. `setRequestValidityMiddleware`
6. `setUploadForwardingMiddleware`
7. `setBucketForwardingMiddleware`

### 2.1 addCustomHeadersMiddleware

文件：[minio/cmd/generic-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/generic-handlers.go:549)

```go
549:func addCustomHeadersMiddleware(h http.Handler) http.Handler {
561:    w.Header().Set(xhttp.AmzRequestID, mustGetRequestID(UTCNow()))
563:    w.Header().Set(xhttp.AmzRequestHostID, globalLocalNodeNameHex)
565:    h.ServeHTTP(w, r)
}
```

作用：

- 注入 `x-amz-request-id`
- 注入 `x-amz-id-2`
- 设置一些安全相关响应头

### 2.2 setAuthMiddleware

文件：[minio/cmd/auth-handler.go](/D:/1-code/__trash__/fs3/minio/cmd/auth-handler.go:616)

```go
616:func setAuthMiddleware(h http.Handler) http.Handler {
621:    aType := getRequestAuthType(r)
625:    amzDate, errCode := parseAmzDateHeader(r)
643:    if curTime.Sub(amzDate) > globalMaxSkewTime || amzDate.Sub(curTime) > globalMaxSkewTime {
654:    h.ServeHTTP(w, r)
}
```

作用：

- 识别签名类型
- 校验日期头
- 校验时钟偏移
- 对不支持的签名直接拒绝

## 3. S3 路由匹配

### 3.1 API Router 注册

文件：[minio/cmd/api-router.go](/D:/1-code/__trash__/fs3/minio/cmd/api-router.go:255)

```go
255:func registerAPIRouter(router *mux.Router) {
257:    api := objectAPIHandlers{
258:        ObjectAPI: newObjectLayerFn,
259:    }
262:    apiRouter := router.PathPrefix(SlashSeparator).Subrouter()
289:    routers = append(routers, apiRouter.PathPrefix("/{bucket}").Subrouter())
}
```

说明：

- `objectAPIHandlers` 持有 `ObjectAPI: newObjectLayerFn`
- 支持 virtual-host style 和 path-style
- path-style 入口是 `/{bucket}/{object}`

### 3.2 PutObject 路由

文件：[minio/cmd/api-router.go](/D:/1-code/__trash__/fs3/minio/cmd/api-router.go:397)

```go
397:        // PutObject
398:        router.Methods(http.MethodPut).Path("/{object:.+}").
399:            HandlerFunc(s3APIMiddleware(api.PutObjectHandler, traceHdrsS3HFlag))
```

说明：

- 普通 PutObject 最终进入 `api.PutObjectHandler`
- 但在它之前，`CopyObject`、`PutObjectPart`、`PutObjectRetention` 等有更高优先级路由

## 4. S3 专用中间件

文件：[minio/cmd/api-router.go](/D:/1-code/__trash__/fs3/minio/cmd/api-router.go:210)

```go
210:func s3APIMiddleware(f http.HandlerFunc, flags ...s3HFlag) http.HandlerFunc {
225:    tracedHandler = httpTraceHdrs(f)
233:    gzippedHandler = gzipHandler(gzippedHandler)
240:    throttledHandler = maxClients(throttledHandler)
245:    statsCollectedHandler := collectAPIStats(handlerName, throttledHandler)
248:    statsCollectedHandler(w, r)
}
```

PutObject 路径上这里的顺序是：

1. `httpTraceHdrs(f)`
2. `gzipHandler(...)`
3. `maxClients(...)`
4. `collectAPIStats(...)`
5. `PutObjectHandler`

### 4.1 maxClients

文件：[minio/cmd/handler-api.go](/D:/1-code/__trash__/fs3/minio/cmd/handler-api.go:309)

```go
309:func maxClients(f http.HandlerFunc) http.HandlerFunc {
312:    globalHTTPStats.incS3RequestsIncoming()
329:    pool := globalAPIConfig.getRequestsPool()
332:    f.ServeHTTP(w, r)
}
```

作用：

- 统计 S3 请求
- 使用请求池限制并发

## 5. PutObjectHandler

文件：[minio/cmd/object-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/object-handlers.go:1793)

```go
1793:func (api objectAPIHandlers) PutObjectHandler(w http.ResponseWriter, r *http.Request) {
1794:    ctx := newContext(r, w, "PutObject")
1797:    objectAPI := api.ObjectAPI()
1803:    vars := mux.Vars(r)
1804:    bucket := vars["bucket"]
1805:    object, err := unescapePath(vars["object"])
1880:    putObject = objectAPI.PutObject
1951:    opts, err = putOptsFromReq(ctx, r, bucket, object, metadata)
2014:    pReader := NewPutObjReader(rawReader)
2070:    reader, objectEncryptionKey, err = EncryptRequest(hashReader, r, bucket, object, metadata)
2088:    pReader, err = pReader.WithEncryption(hashReader, &objectEncryptionKey)
2112:    objInfo, err := putObject(ctx, bucket, object, pReader, opts)
}
```

### 5.1 主要逻辑块

#### 5.1.1 获取对象层

文件：[minio/cmd/object-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/object-handlers.go:1797)

```go
1797:    objectAPI := api.ObjectAPI()
```

说明：

- `api.ObjectAPI` 是 `newObjectLayerFn`
- 这里取到全局对象层实现

#### 5.1.2 解析 bucket / object

文件：[minio/cmd/object-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/object-handlers.go:1803)

```go
1803:    vars := mux.Vars(r)
1804:    bucket := vars["bucket"]
1805:    object, err := unescapePath(vars["object"])
```

#### 5.1.3 基础请求校验

文件：[minio/cmd/object-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/object-handlers.go:1811)

关键函数：

- `etag.FromContentMD5()`，行 `1825`
- `getRequestAuthType()`，行 `1833`
- `newSignV4ChunkedReader()`，行 `1892`
- `newUnsignedV4ChunkedReader()`，行 `1899`
- `isReqAuthenticatedV2()`，行 `1905`
- `reqSignatureV4Verify()`，行 `1912`

#### 5.1.4 元数据与标签提取

文件：[minio/cmd/object-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/object-handlers.go:1860)

关键函数：

- `extractMetadataFromReq()`，行 `1860`
- `tags.ParseObjectTags()`，行 `1867`

#### 5.1.5 权限与配额

文件：[minio/cmd/object-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/object-handlers.go:1883)

关键函数：

- `isPutActionAllowed()`，行 `1884`
- `enforceBucketQuotaHard()`，行 `1927`

#### 5.1.6 生成 ObjectOptions

文件：[minio/cmd/object-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/object-handlers.go:1950)

关键函数：

- `putOptsFromReq()`，行 `1951`
- `hash.NewReaderWithOpts()`，行 `1993`
- `hashReader.AddChecksum()`，行 `2006`
- `NewPutObjReader()`，行 `2014`

#### 5.1.7 对象锁和保留策略

文件：[minio/cmd/object-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/object-handlers.go:2032)

关键函数：

- `isPutActionAllowed(... policy.PutObjectRetentionAction)`，行 `2032`
- `isPutActionAllowed(... policy.PutObjectLegalHoldAction)`，行 `2033`
- `checkPutObjectLockAllowed()`，行 `2037`

#### 5.1.8 SSE 加密

文件：[minio/cmd/object-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/object-handlers.go:2053)

关键函数：

- `EncryptRequest()`，行 `2070`
- `hash.NewReader()`，行 `2083`
- `pReader.WithEncryption()`，行 `2088`

#### 5.1.9 调用对象层

文件：[minio/cmd/object-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/object-handlers.go:2112)

```go
2112:    objInfo, err := putObject(ctx, bucket, object, pReader, opts)
```

这里的 `putObject` 来自：

文件：[minio/cmd/object-handlers.go](/D:/1-code/__trash__/fs3/minio/cmd/object-handlers.go:1880)

```go
1880:    putObject = objectAPI.PutObject
```

## 6. ObjectLayer 接口

文件：[minio/cmd/object-api-interface.go](/D:/1-code/__trash__/fs3/minio/cmd/object-api-interface.go:246)

```go
246:type ObjectLayer interface {
279:    PutObject(ctx context.Context, bucket, object string, data *PutObjReader, opts ObjectOptions) (objInfo ObjectInfo, err error)
}
```

## 7. 对象层实现：erasureServerPools

文件：[minio/cmd/erasure-server-pool.go](/D:/1-code/__trash__/fs3/minio/cmd/erasure-server-pool.go:1085)

```go
1085:func (z *erasureServerPools) PutObject(ctx context.Context, bucket string, object string, data *PutObjReader, opts ObjectOptions) (ObjectInfo, error) {
1087:    if err := checkPutObjectArgs(ctx, bucket, object); err != nil {
1091:    object = encodeDirObject(object)
1100:    idx, err := z.getPoolIdx(ctx, bucket, object, data.Size())
1114:    return z.serverPools[idx].PutObject(ctx, bucket, object, data, opts)
}
```

作用：

1. 校验输入
2. 对目录对象编码
3. 根据对象选择目标 pool
4. 转发到具体 `erasureObjects.PutObject`

## 8. 具体对象写入：erasureObjects.PutObject

文件：[minio/cmd/erasure-object.go](/D:/1-code/__trash__/fs3/minio/cmd/erasure-object.go:1249)

```go
1249:func (er erasureObjects) PutObject(ctx context.Context, bucket string, object string, data *PutObjReader, opts ObjectOptions) (objInfo ObjectInfo, err error) {
1250:    return er.putObject(ctx, bucket, object, data, opts)
}
```

核心在 `putObject()`。

### 8.1 erasureObjects.putObject

文件：[minio/cmd/erasure-object.go](/D:/1-code/__trash__/fs3/minio/cmd/erasure-object.go:1254)

```go
1254:func (er erasureObjects) putObject(ctx context.Context, bucket string, object string, r *PutObjReader, opts ObjectOptions) (objInfo ObjectInfo, err error) {
1263:    ns := er.NewNSLock(bucket, object)
1264:    lkctx, err := ns.GetLock(ctx, globalOperationTimeout)
1296:    storageDisks := er.getDisks()
1369:    onlineDisks, partsMetadata = shuffleDisksAndPartsMetadata(storageDisks, partsMetadata, fi)
1371:    erasure, err := NewErasure(ctx, fi.Erasure.DataBlocks, fi.Erasure.ParityBlocks, fi.Erasure.BlockSize)
1422:    writers[i] = newBitrotWriter(disk, bucket, minioMetaTmpBucket, tempErasureObj, shardFileSize, DefaultBitrotAlgorithm, erasure.ShardSize())
1440:    n, erasureErr := erasure.Encode(ctx, toEncode, writers, buffer, writeQuorum)
1441:    closeErrs := closeBitrotWriters(writers)
1564:    onlineDisks, versions, oldDataDir, err := renameData(ctx, onlineDisks, minioMetaTmpBucket, tempObj, partsMetadata, bucket, object, writeQuorum)
1577:    if err = er.commitRenameDataDir(ctx, bucket, object, oldDataDir, onlineDisks, writeQuorum); err != nil {
}
```

这一层是普通 PutObject 的核心。

### 8.2 关键步骤拆分

#### 8.2.1 加锁

文件：[minio/cmd/erasure-object.go](/D:/1-code/__trash__/fs3/minio/cmd/erasure-object.go:1263)

```go
1263:    ns := er.NewNSLock(bucket, object)
1264:    lkctx, err := ns.GetLock(ctx, globalOperationTimeout)
1269:    defer ns.Unlock(lkctx)
```

#### 8.2.2 选择磁盘与生成元数据

文件：[minio/cmd/erasure-object.go](/D:/1-code/__trash__/fs3/minio/cmd/erasure-object.go:1296)

关键动作：

- `er.getDisks()`，行 `1296`
- 计算 parity / data drive，行 `1298-1341`
- `newFileInfo(...)`，行 `1346`
- `fi.DataDir = mustGetUUID()`，行 `1352`
- `tempObj := uniqueID`，行 `1360`

说明：

- 上传时先写入临时目录
- 最终再原子提交到正式对象位置

#### 8.2.3 构造纠删码器

文件：[minio/cmd/erasure-coding.go](/D:/1-code/__trash__/fs3/minio/cmd/erasure-coding.go:42)

```go
42:func NewErasure(ctx context.Context, dataBlocks, parityBlocks int, blockSize int64) (e Erasure, err error) {
63:    e, err := reedsolomon.New(dataBlocks, parityBlocks, reedsolomon.WithAutoGoroutines(int(e.ShardSize())))
}
```

#### 8.2.4 构造每个磁盘的 writer

文件：[minio/cmd/erasure-object.go](/D:/1-code/__trash__/fs3/minio/cmd/erasure-object.go:1422)

```go
1422:    writers[i] = newBitrotWriter(disk, bucket, minioMetaTmpBucket, tempErasureObj, shardFileSize, DefaultBitrotAlgorithm, erasure.ShardSize())
```

这里把每个 shard 的写入委托给 `newBitrotWriter()`。

## 9. 纠删码写入过程

### 9.1 Erasure.Encode

文件：[minio/cmd/erasure-encode.go](/D:/1-code/__trash__/fs3/minio/cmd/erasure-encode.go:69)

```go
69:func (e *Erasure) Encode(ctx context.Context, src io.Reader, writers []io.Writer, buf []byte, quorum int) (total int64, err error) {
78:    n, err := io.ReadFull(src, buf)
95:    blocks, err = e.EncodeData(ctx, buf[:n])
100:   if err = writer.Write(ctx, blocks); err != nil {
104:   total += int64(n)
}
```

作用：

1. 从客户端数据流读取内容
2. 编码成 data/parity blocks
3. 写入所有 shard writer

### 9.2 multiWriter.Write

文件：[minio/cmd/erasure-encode.go](/D:/1-code/__trash__/fs3/minio/cmd/erasure-encode.go:34)

```go
34:func (p *multiWriter) Write(ctx context.Context, blocks [][]byte) error {
44:    n, p.errs[i] = p.writers[i].Write(blocks[i])
64:    writeErr := reduceWriteQuorumErrs(ctx, p.errs, objectOpIgnoredErrs, p.writeQuorum)
}
```

说明：

- 对每个磁盘 writer 调用 `Write()`
- 不满足 write quorum 就失败

## 10. 临时 shard 写入存储层

### 10.1 newBitrotWriter

文件：[minio/cmd/bitrot.go](/D:/1-code/__trash__/fs3/minio/cmd/bitrot.go:105)

```go
105:func newBitrotWriter(disk StorageAPI, origvolume, volume, filePath string, length int64, algo BitrotAlgorithm, shardSize int64) io.Writer {
107:    return newStreamingBitrotWriter(disk, origvolume, volume, filePath, length, algo, shardSize)
}
```

### 10.2 newStreamingBitrotWriter

文件：[minio/cmd/bitrot-streaming.go](/D:/1-code/__trash__/fs3/minio/cmd/bitrot-streaming.go:108)

```go
108:func newStreamingBitrotWriter(disk StorageAPI, origvolume, volume, filePath string, length int64, algo BitrotAlgorithm, shardSize int64) io.Writer {
122:    go func() {
130:        rb.CloseWithError(disk.CreateFile(context.TODO(), origvolume, volume, filePath, totalFileSize, rb))
131:    }()
132:    return bw
}
```

这是“真正开始写磁盘文件”的第一跳。

### 10.3 streamingBitrotWriter.Write

文件：[minio/cmd/bitrot-streaming.go](/D:/1-code/__trash__/fs3/minio/cmd/bitrot-streaming.go:44)

```go
44:func (b *streamingBitrotWriter) Write(p []byte) (int, error) {
58:    b.h.Write(p)
60:    _, err := b.iow.Write(hashBytes)
65:    n, err := b.iow.Write(p)
}
```

作用：

- 先写 shard 的 bitrot hash
- 再写 shard 本体
- 数据流最终流向 `disk.CreateFile(...)`

## 11. StorageAPI 抽象

文件：[minio/cmd/storage-interface.go](/D:/1-code/__trash__/fs3/minio/cmd/storage-interface.go:29)

```go
29:type StorageAPI interface {
89:    RenameData(ctx context.Context, srcVolume, srcPath string, fi FileInfo, dstVolume, dstPath string, opts RenameOptions) (RenameDataResp, error)
95:    CreateFile(ctx context.Context, origvolume, olume, path string, size int64, reader io.Reader) error
109:   WriteAll(ctx context.Context, volume string, path string, b []byte) (err error)
}
```

普通 PutObject 最终会用到的关键接口是：

1. `CreateFile`
2. `RenameData`
3. `WriteAll`

## 12. 本地磁盘写入路径

### 12.1 DiskID wrapper

文件：[minio/cmd/object-api-common.go](/D:/1-code/__trash__/fs3/minio/cmd/object-api-common.go:61)

```go
61:func newStorageAPI(endpoint Endpoint, opts storageOpts) (storage StorageAPI, err error) {
63:    storage, err := newXLStorage(endpoint, opts.cleanUp)
67:    return newXLStorageDiskIDCheck(storage, opts.healthCheck), nil
}
```

说明：

- 本地盘不会直接暴露 `xlStorage`
- 先包一层 `xlStorageDiskIDCheck`

### 12.2 xlStorageDiskIDCheck.CreateFile

文件：[minio/cmd/xl-storage-disk-id-check.go](/D:/1-code/__trash__/fs3/minio/cmd/xl-storage-disk-id-check.go:437)

```go
437:func (p *xlStorageDiskIDCheck) CreateFile(ctx context.Context, origvolume, volume, path string, size int64, reader io.Reader) (err error) {
438:    ctx, done, err := p.TrackDiskHealth(ctx, storageMetricCreateFile, volume, path)
444:    return p.storage.CreateFile(ctx, origvolume, volume, path, size, io.NopCloser(reader))
}
```

### 12.3 xlStorage.CreateFile

文件：[minio/cmd/xl-storage.go](/D:/1-code/__trash__/fs3/minio/cmd/xl-storage.go:2092)

```go
2092:func (s *xlStorage) CreateFile(ctx context.Context, origvolume, volume, path string, fileSize int64, r io.Reader) (err error) {
2128:    return s.writeAllDirect(ctx, filePath, fileSize, r, os.O_CREATE|os.O_WRONLY|os.O_EXCL, volumeDir, false)
}
```

### 12.4 xlStorage.writeAllDirect

文件：[minio/cmd/xl-storage.go](/D:/1-code/__trash__/fs3/minio/cmd/xl-storage.go:2131)

```go
2131:func (s *xlStorage) writeAllDirect(ctx context.Context, filePath string, fileSize int64, r io.Reader, flags int, skipParent string, truncate bool) (err error) {
2143:    if err = mkdirAll(parentFilePath, 0o777, skipParent); err != nil {
2153:    w, err = OpenFile(filePath, flags, 0o666)
2173:    written, err = io.CopyBuffer(diskHealthWriter(ctx, w), r, *bufp)
2195:    if err = Fdatasync(w); err != nil {
2208:    return w.Close()
}
```

这是真正把 shard 临时文件写入本地文件系统的位置。

### 12.5 写入语义

`writeAllDirect()` 的行为是：

1. 创建父目录
2. 打开目标文件
3. 从 reader 持续拷贝数据到文件
4. `Fdatasync()`
5. `Close()`

也就是说，PutObject 并不是直接写正式对象目录，而是先写到临时目录中的 shard 文件。

## 13. 远端磁盘写入路径

如果某个 `StorageAPI` 是远端磁盘，则调用链变为 RPC。

### 13.1 storageRESTClient.CreateFile

文件：[minio/cmd/storage-rest-client.go](/D:/1-code/__trash__/fs3/minio/cmd/storage-rest-client.go:392)

```go
392:func (client *storageRESTClient) CreateFile(ctx context.Context, origvolume, volume, path string, size int64, reader io.Reader) error {
399:    respBody, err := client.call(ctx, storageRESTMethodCreateFile, values, io.NopCloser(reader), size)
404:    _, err = waitForHTTPResponse(respBody)
}
```

### 13.2 storageRESTServer.CreateFileHandler

文件：[minio/cmd/storage-rest-server.go](/D:/1-code/__trash__/fs3/minio/cmd/storage-rest-server.go:336)

```go
336:func (s *storageRESTServer) CreateFileHandler(w http.ResponseWriter, r *http.Request) {
353:    done(s.getStorage().CreateFile(r.Context(), origvolume, volume, filePath, int64(fileSize), body))
}
```

### 13.3 远端最终仍落到 xlStorage.CreateFile

最终远端节点仍然执行：

- `xlStorageDiskIDCheck.CreateFile`
- `xlStorage.CreateFile`
- `xlStorage.writeAllDirect`

所以本地盘和远端盘在最后落盘逻辑上一致，只是远端先经过 storage REST RPC。

## 14. 提交阶段：从临时对象切换到正式对象

### 14.1 renameData

文件：[minio/cmd/erasure-object.go](/D:/1-code/__trash__/fs3/minio/cmd/erasure-object.go:1019)

```go
1019:func renameData(ctx context.Context, disks []StorageAPI, srcBucket, srcEntry string, metadata []FileInfo, dstBucket, dstEntry string, writeQuorum int) ([]StorageAPI, []byte, string, error) {
1046:    resp, err := disks[index].RenameData(ctx, srcBucket, srcEntry, fi, dstBucket, dstEntry, RenameOptions{})
}
```

说明：

- 前面写的是 `minioMetaTmpBucket/tempObj/...`
- 这里开始把临时对象提交到最终 `bucket/object`

### 14.2 本地盘 RenameData wrapper

文件：[minio/cmd/xl-storage-disk-id-check.go](/D:/1-code/__trash__/fs3/minio/cmd/xl-storage-disk-id-check.go:483)

```go
483:func (p *xlStorageDiskIDCheck) RenameData(ctx context.Context, srcVolume, srcPath string, fi FileInfo, dstVolume, dstPath string, opts RenameOptions) (res RenameDataResp, err error) {
```

### 14.3 远端盘 RenameData RPC

客户端：

文件：[minio/cmd/storage-rest-client.go](/D:/1-code/__trash__/fs3/minio/cmd/storage-rest-client.go:485)

```go
485:func (client *storageRESTClient) RenameData(ctx context.Context, srcVolume, srcPath string, fi FileInfo,
499:    resp, err = storageRenameDataRPC.Call(ctx, client.gridConn, &params)
```

服务端：

文件：[minio/cmd/storage-rest-server.go](/D:/1-code/__trash__/fs3/minio/cmd/storage-rest-server.go:703)

```go
703:func (s *storageRESTServer) RenameDataHandler(p *RenameDataHandlerParams) (*RenameDataResp, *grid.RemoteErr) {
708:    resp, err := s.getStorage().RenameData(context.Background(), p.SrcVolume, p.SrcPath, p.FI, p.DstVolume, p.DstPath, p.Opts)
}
```

## 15. xlStorage.RenameData

文件：[minio/cmd/xl-storage.go](/D:/1-code/__trash__/fs3/minio/cmd/xl-storage.go:2557)

这是 PutObject “提交事务”的关键函数。

```go
2557:func (s *xlStorage) RenameData(ctx context.Context, srcVolume, srcPath string, fi FileInfo, dstVolume, dstPath string, opts RenameOptions) (res RenameDataResp, err error) {
2604:    srcFilePath := pathutil.Join(srcVolumeDir, pathJoin(srcPath, xlStorageFormatFile))
2605:    dstFilePath := pathutil.Join(dstVolumeDir, pathJoin(dstPath, xlStorageFormatFile))
2614:    srcDataPath = retainSlash(pathJoin(srcVolumeDir, srcPath, dataDir))
2618:    dstDataPath = pathutil.Join(dstVolumeDir, dstPath, dataDir)
2803:    if err = xlMeta.AddVersion(fi); err != nil {
2823:    newDstBuf, err := xlMeta.AppendTo(metaDataPoolGet())
2836:    if err = s.WriteAll(ctx, srcVolume, pathJoin(srcPath, xlStorageFormatFile), newDstBuf); err != nil {
2862:    if err = renameAll(srcDataPath, dstDataPath, skipParent); err != nil {
2896:    if err = renameAll(srcFilePath, dstFilePath, skipParent); err != nil {
2915:    return res, nil
}
```

### 15.1 这个函数做了什么

它不是简单 rename 一个文件，而是完成以下事务：

1. 读取目标位置已有的 `xl.meta`
2. 解析/保留旧版本信息
3. 把新上传对象作为一个新版本加入 `xl.meta`
4. 先把新的 `xl.meta` 写回临时目录
5. 把临时 data dir rename 到正式对象目录
6. 最后把临时 `xl.meta` rename 到正式对象目录

这里说明 MinIO PutObject 的正式提交是：

- 先写 shard 到临时目录
- 再通过 `RenameData()` 做“元数据 + 数据目录”的原子切换

### 15.2 提交中的 WriteAll

文件：[minio/cmd/xl-storage.go](/D:/1-code/__trash__/fs3/minio/cmd/xl-storage.go:2305)

```go
2305:func (s *xlStorage) WriteAll(ctx context.Context, volume string, path string, b []byte) (err error) {
2319:    return s.writeAll(ctx, volume, path, b, true, volumeDir)
}
```

用途：

- 在 `RenameData()` 中把新的 `xl.meta` 先写回临时位置

### 15.3 RenameData 里的关键提交顺序

文件：[minio/cmd/xl-storage.go](/D:/1-code/__trash__/fs3/minio/cmd/xl-storage.go:2836)

```go
2836:if err = s.WriteAll(ctx, srcVolume, pathJoin(srcPath, xlStorageFormatFile), newDstBuf); err != nil {
2862:if err = renameAll(srcDataPath, dstDataPath, skipParent); err != nil {
2896:if err = renameAll(srcFilePath, dstFilePath, skipParent); err != nil {
```

顺序是：

1. `WriteAll()` 写临时 `xl.meta`
2. `renameAll(srcDataPath, dstDataPath, ...)` 提交数据目录
3. `renameAll(srcFilePath, dstFilePath, ...)` 提交 `xl.meta`

这个顺序是 PutObject 存储语义的核心。

## 16. 提交后清理旧数据目录

文件：[minio/cmd/erasure-object.go](/D:/1-code/__trash__/fs3/minio/cmd/erasure-object.go:1836)

```go
1836:func (er erasureObjects) commitRenameDataDir(ctx context.Context, bucket, object, dataDir string, onlineDisks []StorageAPI, writeQuorum int) error {
1846:    return onlineDisks[index].Delete(ctx, bucket, pathJoin(object, dataDir), DeleteOptions{
1847:        Recursive: true,
1848:    })
}
```

作用：

- 当覆盖旧对象版本时，清理旧 data dir

## 17. 最终完整调用栈

### 17.1 逻辑调用链

1. `xhttp.Server.UseHandler(...)`
2. `setCriticalErrorHandler(...)`
3. `corsHandler(...)`
4. `configureServerHandler()`
5. `registerAPIRouter()`
6. `router.Methods(PUT).Path("/{object:.+}")`
7. `s3APIMiddleware(api.PutObjectHandler, traceHdrsS3HFlag)`
8. `maxClients()`
9. `PutObjectHandler()`
10. `objectAPI.PutObject()`
11. `erasureServerPools.PutObject()`
12. `erasureObjects.PutObject()`
13. `erasureObjects.putObject()`
14. `NewErasure()`
15. `Erasure.Encode()`
16. `multiWriter.Write()`
17. `newBitrotWriter()`
18. `newStreamingBitrotWriter()`
19. `StorageAPI.CreateFile()`
20. `xlStorageDiskIDCheck.CreateFile()` 或 `storageRESTClient.CreateFile()`
21. `xlStorage.CreateFile()`
22. `xlStorage.writeAllDirect()`
23. `renameData()`
24. `StorageAPI.RenameData()`
25. `xlStorageDiskIDCheck.RenameData()` 或 `storageRESTClient.RenameData()`
26. `xlStorage.RenameData()`
27. `WriteAll()` 写新的临时 `xl.meta`
28. `renameAll(srcDataPath, dstDataPath, ...)`
29. `renameAll(srcFilePath, dstFilePath, ...)`
30. `commitRenameDataDir()`

### 17.2 落盘关键点

真正涉及磁盘写入的关键位置：

1. `xlStorage.writeAllDirect()`  
   文件：[minio/cmd/xl-storage.go](/D:/1-code/__trash__/fs3/minio/cmd/xl-storage.go:2131)

2. `xlStorage.WriteAll()`  
   文件：[minio/cmd/xl-storage.go](/D:/1-code/__trash__/fs3/minio/cmd/xl-storage.go:2305)

3. `xlStorage.RenameData()`  
   文件：[minio/cmd/xl-storage.go](/D:/1-code/__trash__/fs3/minio/cmd/xl-storage.go:2557)

## 18. 结论

MinIO 普通 `PutObject` 的存储流程不是“handler 直接写目标文件”，而是两阶段：

### 阶段一：写临时对象

调用链：

`PutObjectHandler -> ObjectLayer.PutObject -> erasureObjects.putObject -> Erasure.Encode -> StorageAPI.CreateFile -> xlStorage.writeAllDirect`

结果：

- 每块磁盘上在临时目录写入 shard 文件

### 阶段二：提交到正式对象路径

调用链：

`renameData -> StorageAPI.RenameData -> xlStorage.RenameData`

结果：

- 合并并生成新的 `xl.meta`
- data dir 从临时目录切到正式对象目录
- `xl.meta` 从临时目录切到正式对象目录

因此，分析 fs3 对 MinIO 的兼容性时，重点必须对齐以下行为：

1. `PutObject` 先写临时对象
2. 纠删码写入阶段使用 shard writer
3. 最终提交通过 `RenameData()` 完成
4. `xl.meta` 写入与 data dir rename 有固定顺序
5. 覆盖场景下旧 data dir 需要额外清理

## 19. 底层存储补充观察

下面补充这次重新沿调用栈核对后，针对“真正写盘发生在哪里、提交在哪里完成”的更细粒度观察。

## 19.1 真正开始写磁盘，不是在 handler，而是在 shard writer 启动后

虽然 `PutObjectHandler()` 最后调用的是：

`putObject(ctx, bucket, object, pReader, opts)`

但这一步还没有直接写正式对象路径。

真正进入存储层写入的关键点在 `erasureObjects.putObject()` 中构造 writer 的时候：

文件：`minio/cmd/erasure-object.go`

```go
1403:    shardFileSize := erasure.ShardFileSize(data.Size())
1404:    writers := make([]io.Writer, len(onlineDisks))
1422:    writers[i] = newBitrotWriter(disk, bucket, minioMetaTmpBucket, tempErasureObj, shardFileSize, DefaultBitrotAlgorithm, erasure.ShardSize())
```

这里说明：

1. PutObject 不是“直接把对象写成一个文件”
2. 而是先把对象切成 erasure shard
3. 每个在线磁盘拿到一个 shard writer
4. shard writer 的目标路径位于临时 bucket `minioMetaTmpBucket`

## 19.2 `Erasure.Encode()` 负责“读客户端流 -> 切块 -> 分发到各盘 writer”

文件：`minio/cmd/erasure-encode.go`

```go
69:func (e *Erasure) Encode(ctx context.Context, src io.Reader, writers []io.Writer, buf []byte, quorum int) (total int64, err error) {
78:    n, err := io.ReadFull(src, buf)
95:    blocks, err = e.EncodeData(ctx, buf[:n])
100:   if err = writer.Write(ctx, blocks); err != nil {
```

以及：

```go
34:func (p *multiWriter) Write(ctx context.Context, blocks [][]byte) error {
44:    n, p.errs[i] = p.writers[i].Write(blocks[i])
64:    writeErr := reduceWriteQuorumErrs(ctx, p.errs, objectOpIgnoredErrs, p.writeQuorum)
```

这一步的真实语义是：

1. 从客户端 reader 持续读取数据
2. 编码成 data/parity blocks
3. 把每个 block 发给对应磁盘 writer
4. 如果不满足 write quorum，则整个写入失败

因此，从存储视角看，客户端对象数据并不是“一次性下沉到 storage”，而是经过纠删码分块后逐批写到各盘。

## 19.3 `streamingBitrotWriter` 不是裸写 shard，而是“hash + shard”串流写入

文件：`minio/cmd/bitrot-streaming.go`

```go
44:func (b *streamingBitrotWriter) Write(p []byte) (int, error) {
58:    b.h.Write(p)
59:    hashBytes := b.h.Sum(nil)
60:    _, err := b.iow.Write(hashBytes)
65:    n, err := b.iow.Write(p)
```

说明：

1. 每次写一个 shard block 时
2. 先计算该 block 的 bitrot hash
3. 先写 hash
4. 再写 shard block 本体

也就是说，临时 shard 文件并不是“纯 shard 数据文件”，而是带 bitrot 校验数据的流式格式。

## 19.4 `newStreamingBitrotWriter()` 里第一次真正落到 StorageAPI

文件：`minio/cmd/bitrot-streaming.go`

```go
108:func newStreamingBitrotWriter(disk StorageAPI, origvolume, volume, filePath string, length int64, algo BitrotAlgorithm, shardSize int64) io.Writer {
122:    go func() {
130:        rb.CloseWithError(disk.CreateFile(context.TODO(), origvolume, volume, filePath, totalFileSize, rb))
131:    }()
```

这说明“真正开始写某块磁盘”的第一跳是：

`newStreamingBitrotWriter() -> StorageAPI.CreateFile(...)`

关键点：

1. writer 的前端是 `streamingBitrotWriter`
2. 后端通过 ringbuffer 异步连接到 `disk.CreateFile(...)`
3. 所以 MinIO 是边编码边通过管道把数据流给 storage 层
4. 并不是先把完整 shard 缓存在上层再一次性落盘

## 19.5 本地磁盘最终落盘点是 `xlStorage.writeAllDirect()`

本地盘调用链为：

1. `StorageAPI.CreateFile()`
2. `xlStorageDiskIDCheck.CreateFile()`
3. `xlStorage.CreateFile()`
4. `xlStorage.writeAllDirect()`

文件：`minio/cmd/xl-storage.go`

```go
2092:func (s *xlStorage) CreateFile(ctx context.Context, origvolume, volume, path string, fileSize int64, r io.Reader) (err error) {
2128:    return s.writeAllDirect(ctx, filePath, fileSize, r, os.O_CREATE|os.O_WRONLY|os.O_EXCL, volumeDir, false)
```

真正 OS 级写盘的位置：

```go
2131:func (s *xlStorage) writeAllDirect(ctx context.Context, filePath string, fileSize int64, r io.Reader, flags int, skipParent string, truncate bool) (err error) {
2143:    if err = mkdirAll(parentFilePath, 0o777, skipParent); err != nil {
2151:        w, err = OpenFileDirectIO(filePath, flags, 0o666)
2153:        w, err = OpenFile(filePath, flags, 0o666)
2171:        written, err = xioutil.CopyAligned(diskHealthWriter(ctx, w), r, *bufp, fileSize, w)
2173:        written, err = io.CopyBuffer(diskHealthWriter(ctx, w), r, *bufp)
2195:    if err = Fdatasync(w); err != nil {
2208:    return w.Close()
}
```

因此，本地存储层真正的落盘语义是：

1. 创建父目录
2. 打开目标文件
3. 从 reader 持续复制数据到文件
4. `Fdatasync`
5. `Close`

这里写入的不是正式对象路径，而是临时路径下的 shard 文件。

## 19.6 远端磁盘不会改变最终落盘语义，只是多一层 storage REST

远端盘调用链：

客户端：

```go
392:func (client *storageRESTClient) CreateFile(ctx context.Context, origvolume, volume, path string, size int64, reader io.Reader) error {
399:    respBody, err := client.call(ctx, storageRESTMethodCreateFile, values, io.NopCloser(reader), size)
```

服务端：

```go
336:func (s *storageRESTServer) CreateFileHandler(w http.ResponseWriter, r *http.Request) {
353:    done(s.getStorage().CreateFile(r.Context(), origvolume, volume, filePath, int64(fileSize), body))
```

结论：

1. 远端磁盘写入先经过 storage REST
2. 但远端节点最终仍执行自己的 `StorageAPI.CreateFile()`
3. 最后仍会落到远端节点的 `xlStorage.CreateFile() -> writeAllDirect()`

因此，本地盘和远端盘在最后的物理写盘语义上是一致的。

## 19.7 `renameData()` 才是从临时对象切换到正式对象的提交点

文件：`minio/cmd/erasure-object.go`

```go
1564:    onlineDisks, versions, oldDataDir, err := renameData(ctx, onlineDisks, minioMetaTmpBucket, tempObj, partsMetadata, bucket, object, writeQuorum)
1577:    if err = er.commitRenameDataDir(ctx, bucket, object, oldDataDir, onlineDisks, writeQuorum); err != nil {
```

说明：

1. `CreateFile()` 只是把 shard 写到临时路径
2. 真正提交到 `bucket/object` 是在 `renameData()`
3. 覆盖旧版本后的旧 `dataDir` 清理由 `commitRenameDataDir()` 另行完成

## 19.8 `xlStorage.RenameData()` 的提交顺序是存储兼容性的核心

文件：`minio/cmd/xl-storage.go`

```go
2836:if err = s.WriteAll(ctx, srcVolume, pathJoin(srcPath, xlStorageFormatFile), newDstBuf); err != nil {
2862:if err = renameAll(srcDataPath, dstDataPath, skipParent); err != nil {
2896:if err = renameAll(srcFilePath, dstFilePath, skipParent); err != nil {
```

从底层存储角度，顺序非常明确：

1. 先把新的 `xl.meta` 写入临时源路径
2. 再把 data dir 从临时路径 rename 到正式对象路径
3. 最后把 `xl.meta` 从临时路径 rename 到正式对象路径

这说明 MinIO PutObject 的“正式可见性”不是来自 `CreateFile()`，而是来自 `RenameData()`。

## 19.9 覆盖写旧数据目录的删除在提交之后单独进行

文件：`minio/cmd/erasure-object.go`

```go
1836:func (er erasureObjects) commitRenameDataDir(ctx context.Context, bucket, object, dataDir string, onlineDisks []StorageAPI, writeQuorum int) error {
1846:    return onlineDisks[index].Delete(ctx, bucket, pathJoin(object, dataDir), DeleteOptions{
1847:        Recursive: true,
1848:    })
```

说明：

1. 覆盖写时旧 `dataDir` 不会在 `CreateFile()` 阶段处理
2. 也不是在 `renameAll(srcDataPath, dstDataPath)` 同时处理
3. 而是在提交成功之后，再按 disk 批量递归删除旧 data dir

## 19.10 底层存储视角下的最终结论

如果只看 MinIO PutObject 的底层存储语义，可以归纳成以下几点：

1. 写入单位不是完整对象文件，而是每盘一个 shard 文件
2. shard 写入内容不是裸数据，而是 `bitrot hash + shard data`
3. 真正 OS 级写盘发生在 `xlStorage.writeAllDirect()`
4. 数据先落在临时 bucket / 临时对象路径，不直接进入正式 `bucket/object`
5. 正式提交由 `RenameData()` 完成，而不是 `CreateFile()`
6. 覆盖场景下旧 `dataDir` 的删除发生在提交之后的单独清理阶段

因此，若要实现与 MinIO 更接近的 PutObject 存储兼容性，仅仅模仿“写一个文件并写一个元数据”是不够的；至少需要对齐：

1. 临时 shard 写入
2. `CreateFile()` 的真实语义
3. `RenameData()` 的提交顺序
4. 覆盖场景下旧 data dir 的延后清理
