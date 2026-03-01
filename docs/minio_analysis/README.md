# MinIO 协议兼容分析（当前目录版本）

本目录对 `./minio` 源码进行协议兼容性梳理，并按协议类型拆分。

## 文件导航

- `00_protocol_matrix.md`：全量协议矩阵（按类型汇总）
- `01_s3_and_sts.md`：S3 兼容 API 与 STS 协议
- `02_admin_observability_kms.md`：管理、健康检查、指标、KMS API
- `03_file_transfer.md`：FTP/FTPS/SFTP 文件传输协议
- `04_identity_policy_integration.md`：身份认证与授权集成协议（OIDC/LDAP/插件/OPA）
- `05_event_notification_protocols.md`：事件通知目标协议
- `06_internal_cluster_protocols.md`：集群内部通信协议（REST + Grid/WebSocket）
- `07_s3_http_https_all_interfaces_methods.md`：S3 over HTTP/HTTPS 全接口与全方法清单

## 分析范围

- 范围：`D:\1-code\__trash__\fs3\minio`
- 方法：以源码路由、配置结构、目标驱动实现为主，文档为辅。
- 结论口径：
  - “对外协议”：客户端/运维系统可直接接入。
  - “内部协议”：MinIO 节点间或子系统内部通信使用。
