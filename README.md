# s3-mount-gateway

Go 实现的本地目录 S3 网关，支持多挂载类型，并拆分双端口：
- `:3001` 控制面（S3 兼容 + 管理 API + 预签名下载/上传 + 分块上传）
- `:3000` 策略访问面（按策略组控制 bucket/prefix 可访问范围）

## 启动

```powershell
Copy-Item config.example.json config.json
New-Item -ItemType Directory -Force -Path .\data\docs,.\data\readonly,.\data\base,.\data\upper | Out-Null
go run .\cmd\server\main.go -config .\config.json
```

## 路由约定

### 3001 控制面

1. S3 兼容（MinIO 客户端可用）
- 根路径即 S3 路由，例如 `GET /docs/file.txt`

2. 对象增删查
- `PUT /api/object/{bucket}/{key...}` 上传
- `GET /api/object/{bucket}/{key...}` 下载
- `HEAD /api/object/{bucket}/{key...}` 元数据
- `DELETE /api/object/{bucket}/{key...}` 删除
- `GET /api/list/{bucket}?prefix=&delimiter=&token=&max_keys=` 列表

3. 预签名下载/上传
- `POST /api/presign/download`
- `POST /api/presign/upload`

请求体示例：
```json
{
  "bucket": "docs",
  "key": "a/b.txt",
  "expires_seconds": 900
}
```

返回 `url` 后使用：
- 下载：`GET {url}`
- 上传：`PUT {url}`

4. 分块上传（预签名分片）
- `POST /api/multipart/init` -> `{ upload_id }`
- `POST /api/multipart/presign-part` -> 获取某个 `part_number` 的上传 URL
- `PUT {part_url}` -> 上传分片
- `POST /api/multipart/complete` -> 合并写入目标对象
- `DELETE /api/multipart/{upload_id}` -> 中止

### 3000 策略访问面

访问路径：
- `GET /content/{group}/{user}/{bucket}/{object_path...}`
- `HEAD /content/{group}/{user}/{bucket}/{object_path...}`

策略匹配逻辑：
- 策略组必须 `enabled=true`
- 先校验策略组 `users`（如配置）
- 逐条匹配 `rules`（`bucket + prefix + rule.users`），最后一条命中的 `allow` 生效
- 未命中默认拒绝

## MinIO 客户端兼容

```powershell
mc alias set local http://127.0.0.1:3001 minioadmin minioadmin
mc ls local
Set-Content .\demo.txt "hello"
mc cp .\demo.txt local/docs/team-a/demo.txt
mc cat local/docs/team-a/demo.txt
```

## 配置字段

- `listen_policy`: 策略面监听地址，默认 `:3000`
- `listen_control`: 控制面监听地址，默认 `:3001`
- `control_public_base`: 预签名 URL 返回的基础地址
- `mounts`: bucket 挂载（`rw`/`ro`/`overlay`）
- `policy_groups`: 策略组定义
