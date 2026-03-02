# 09 策略引擎架构（基于 MinIO 源码分析）

## 1. 目标

本文梳理 MinIO 策略引擎的核心组件、评估流程和数据结构，为 fs3 实现访问控制提供参考。

## 2. 源码锚点

- IAM 主逻辑：`minio/cmd/iam.go` (`IAMSys`)
- IAM 存储抽象 + 缓存：`minio/cmd/iam-store.go` (`IAMStoreSys`, `iamCache`)
- IAM 对象存储后端：`minio/cmd/iam-object-store.go`
- IAM etcd 后端：`minio/cmd/iam-etcd-store.go`
- 桶策略系统：`minio/cmd/bucket-policy.go` (`PolicySys`)
- 桶策略 HTTP 处理器：`minio/cmd/bucket-policy-handlers.go`
- OPA 集成：`minio/internal/config/policy/opa/config.go`
- AuthZ 插件：`minio/internal/config/policy/plugin/config.go`
- 凭证类型：`minio/internal/auth/credentials.go`

## 3. 核心组件

### 3.1 IAMSys（身份级访问控制）

主入口 `IsAllowed(args policy.Args)` 负责所有请求的策略评估，根据凭证类型分发：

- `IsAllowedSTS(args, parentUser)` — STS 临时凭证
- `IsAllowedServiceAccount(args, parentUser)` — 服务账户
- 普通用户 → `PolicyDBGet()` + `GetCombinedPolicy()` 合并评估

### 3.2 PolicySys（桶级访问控制）

- `IsAllowed(args policy.BucketPolicyArgs)` — 检查桶策略
- `Get(bucket)` — 获取指定桶的策略

### 3.3 外部策略插件

- **OPA**（`internal/config/policy/opa/`）：通过 HTTP 调用外部 OPA 服务器，支持两种响应格式 `{result: bool}` 和 `{result: {allow: bool}}`
- **AuthZ Plugin**（`internal/config/policy/plugin/`）：通用授权插件，同样基于 HTTP 委托

## 4. 评估流程

```
请求进入 IsAllowed()
  ↓
1. 检查是否启用 OPA/AuthZ 插件 → 委托给外部引擎
  ↓
2. 检查是否 owner（root 凭证）→ 直接放行
  ↓
3. 根据凭证类型分发：
   - 普通用户 → 用户策略 + 组策略合并评估
   - STS 临时凭证 → 父用户策略 AND 会话策略
   - ServiceAccount → 父用户策略 AND 会话策略
  ↓
4. 桶策略额外检查
```

访问控制优先级：Owner > OPA/Plugin > IAM 策略（用户+组）> 桶策略

## 5. 存储与缓存

### 5.1 存储后端

`IAMStoreSys` 实现 `IAMStorageAPI` 接口，支持两种后端：

- 对象存储（本地文件系统）
- etcd（分布式场景）

### 5.2 内存缓存（iamCache）

| 缓存字段 | 映射关系 |
|---|---|
| `iamPolicyDocsMap` | 策略名 → PolicyDoc |
| `iamUsersMap` | 用户名 → UserIdentity |
| `iamUserPolicyMap` | 用户名 → MappedPolicy |
| `iamSTSAccountsMap` | STS key → UserIdentity |
| `iamSTSPolicyMap` | STS key → MappedPolicy |
| `iamGroupsMap` | 组名 → GroupInfo |
| `iamUserGroupMemberships` | 用户 → 所属组列表 |
| `iamGroupPolicyMap` | 组名 → MappedPolicy |

## 6. 关键数据结构

### 6.1 PolicyDoc

```go
type PolicyDoc struct {
    Version    int
    Policy     policy.Policy
    CreateDate time.Time
    UpdateDate time.Time
}
```

### 6.2 MappedPolicy

```go
type MappedPolicy struct {
    Version   int
    Policies  string    // 逗号分隔的策略名
    UpdatedAt time.Time
}
```

### 6.3 UserIdentity

```go
type UserIdentity struct {
    Version     int
    Credentials auth.Credentials
    UpdatedAt   time.Time
}
```

## 7. 策略合并逻辑

- 普通用户：获取用户直接关联的策略 + 用户所属所有组的策略，合并为单一 `combinedPolicy` 后调用 `IsAllowed(args)`
- STS / ServiceAccount：会话策略（inline policy）与父用户策略取 AND，即会话策略不能超出父用户权限范围

## 8. 对 fs3 的启示

- 最小实现只需 IAMSys + PolicySys 两层，不需要 OPA/Plugin
- 缓存层是性能关键，策略文档和映射关系应常驻内存
- STS 和 ServiceAccount 的 AND 语义是安全约束的核心，不可省略
- 桶策略独立于 IAM 策略，两者并行评估
