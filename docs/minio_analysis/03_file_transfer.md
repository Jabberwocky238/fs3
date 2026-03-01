# 文件传输协议分析（FTP / FTPS / SFTP）

## 1. 协议范围

文档与源码都显示 MinIO 原生支持三类文件传输协议：

- FTP
- FTPS（FTP over TLS）
- SFTP（SSH File Transfer Protocol）

证据：
- `minio/docs/ftp/README.md:1`, `:18`, `:20`, `:24`, `:26`

## 2. FTP / FTPS 实现

- 服务入口：`startFTPServer(args []string)`
  - 证据：`minio/cmd/ftp-server.go:71`
- 默认端口：`8021`（未显式设置时）
  - 证据：`minio/cmd/ftp-server.go:126`
- TLS 相关参数：
  - `tls-private-key`
  - `tls-public-cert`
  - `force-tls`
  - 证据：`minio/cmd/ftp-server.go:101`, `:103`, `:105`, `:139`
- FTP 库配置开启 `ExplicitFTPS`。
  - 证据：`minio/cmd/ftp-server.go:157`

结论：
- 同一 FTP 服务可在明文 FTP 与显式 TLS（FTPS）模式下工作，且支持强制 TLS。

## 3. SFTP 实现

- 服务入口：`startSFTPServer(args []string)`
  - 证据：`minio/cmd/sftp-server.go:394`
- 默认端口：`8022`（未显式设置时）
  - 证据：`minio/cmd/sftp-server.go:446`
- 关键依赖：`golang.org/x/crypto/ssh`、`github.com/pkg/sftp`
  - 证据：`minio/cmd/sftp-server.go:36`, `:37`
- 支持 SSH 密钥、口令、LDAP 结合等认证路径（由配置与 IAM 状态决定）。
  - 证据：`minio/cmd/sftp-server.go:129`, `:133`, `:141`, `:193`

## 4. 与 S3 协议关系

- FTP/SFTP 是“额外访问协议面”，核心权限模型仍复用 MinIO IAM/S3 权限体系。
- 文档中明确版本对象场景仍建议通过 S3 API 客户端访问。
  - 证据：`minio/docs/ftp/README.md:43`, `:44`

## 5. 结论

- 当前版本 MinIO 具备多协议文件访问能力：FTP、FTPS、SFTP。
- 生产环境建议优先 FTPS/SFTP，明文 FTP 仅用于受控内网或兼容场景。
