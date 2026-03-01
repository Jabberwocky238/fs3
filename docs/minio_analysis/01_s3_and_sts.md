# S3 与 STS 协议分析

## 1. S3 兼容协议面

- MinIO 在项目级别声明为 `S3-compatible`，并强调 `S3 API Compatible`。
  - 证据：`minio/README.md:16`, `minio/README.md:19`
- S3 路由统一由 `registerAPIRouter` 注册。
  - 证据：`minio/cmd/api-router.go:254`

## 2. S3 认证与签名兼容

- 支持 SigV4 与 SigV2 兼容路径，并区分 presigned、streaming、JWT、STS 等请求类型。
  - 证据：`minio/cmd/auth-handler.go:107`, `:117`, `:118`, `:186`, `:596`
- SigV2（兼容）含 presigned 校验逻辑。
  - 证据：`minio/cmd/signature-v2.go:294`
- SigV4 streaming（`aws-chunked`）被显式支持。
  - 证据：`minio/cmd/streaming-signature-v4.go:43`, `:97`

## 3. S3 扩展能力（仍走 S3 API）

- Bucket/Object 通知相关 API（含 `ListenNotification`）已在 API Router 中注册。
  - 证据：`minio/cmd/api-router.go:444`, `:446`, `:632`, `:634`
- Object Lambda、S3 ZIP 等为 S3 语义扩展，协议承载仍是 HTTP(S) + S3 风格接口。
  - 证据：`minio/cmd/object-lambda-handlers.go:136`, `minio/cmd/s3-zip-handlers.go:83`

## 4. STS 协议兼容

- STS API 版本固定校验为 `2011-06-15`（AWS STS Query API 版本）。
  - 证据：`minio/cmd/sts-handlers.go:47`, `:277`
- 支持动作：
  - `AssumeRole`
  - `AssumeRoleWithClientGrants`
  - `AssumeRoleWithWebIdentity`
  - `AssumeRoleWithLDAPIdentity`
  - `AssumeRoleWithCertificate`
  - `AssumeRoleWithCustomToken`
  - 证据：`minio/cmd/sts-handlers.go:161`, `:168`, `:174`, `:181`, `:186`, `:260`

## 5. 结论

- MinIO 的“主协议”是 S3 over HTTP/HTTPS。
- STS 是与 S3 同平面的身份凭证协议层，负责签发临时 AK/SK/SessionToken，供后续 S3 请求使用。
- 该版本在认证兼容上覆盖了传统 SigV2、主流 SigV4、流式 SigV4 和多种 STS 扩展动作。
