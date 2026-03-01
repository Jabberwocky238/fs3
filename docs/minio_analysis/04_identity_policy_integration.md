# 身份认证与授权集成协议分析

## 1. OIDC / OpenID Connect

- STS 文档明确支持 WebIdentity（OIDC 提供方）。
  - 证据：`minio/docs/sts/README.md:18`
- 配置帮助中明确需要 `openid discovery document`（`.well-known/openid-configuration`）。
  - 证据：`minio/internal/config/identity/openid/help.go:37`

协议特征：
- OIDC Discovery + JWT claims 验证
- 与 STS `AssumeRoleWithWebIdentity` 联动

## 2. LDAP / AD

- STS 文档明确支持 AD/LDAP 用户换取临时凭证。
  - 证据：`minio/docs/sts/README.md:19`
- LDAP 配置项包含服务器地址、SRV、Bind DN、StartTLS 等。
  - 证据：`minio/internal/config/identity/ldap/help.go:31`, `:37`, `:44`, `:101`

协议特征：
- LDAP/LDAPS/StartTLS 目录访问
- 与 STS `AssumeRoleWithLDAPIdentity` 联动

## 3. Identity Management Plugin（自定义鉴权 Webhook）

- MinIO 支持通过身份插件 webhook 验证 `AssumeRoleWithCustomToken`。
  - 证据：`minio/docs/iam/identity-management-plugin.md:5`, `:32`
- 插件侧是 HTTP POST 验证流程。
  - 证据：`minio/docs/iam/identity-management-plugin.md:32`

协议特征：
- HTTP/HTTPS Webhook
- MinIO 对 token 采取“opaque token”外部校验模式

## 4. OPA 授权协议

- 文档明确通过 OPA HTTP API 进行授权。
  - 证据：`minio/docs/iam/opa.md:3`
- OPA URL 在配置中通过 `ParseHTTPURL` 校验。
  - 证据：`minio/internal/config/policy/opa/config.go:138`

协议特征：
- HTTP/HTTPS 策略查询
- 可与常规 IAM、STS、LDAP/OIDC 等凭证体系叠加

## 5. STS 扩展动作与身份协议映射

- `AssumeRoleWithWebIdentity` -> OIDC
- `AssumeRoleWithLDAPIdentity` -> LDAP/AD
- `AssumeRoleWithCustomToken` -> Identity Plugin Webhook
- `AssumeRoleWithCertificate` -> 客户端证书身份

证据：
- `minio/cmd/sts-handlers.go:168`, `:174`, `:181`, `:186`

## 6. 结论

- MinIO 身份协议层不是单一协议，而是以 STS 为统一出口，后接 OIDC、LDAP、证书、自定义 Webhook 等多种身份源。
- 授权可本地 IAM，也可外接 OPA HTTP 策略引擎。
