# 架构迁移指南

## 新架构概述

基于 MinIO 的设计，新架构分为两层：

1. **ObjectLayer** - 高层 API（bucket/object 操作）
2. **StorageAPI** - 底层存储接口（volume/file/metadata 操作）

## 核心组件

### 1. Traits

- `ObjectLayer` (`src/types/traits/object_layer.rs`) - 对象层接口
- `StorageAPI` (`src/types/traits/storage_api.rs`) - 存储层接口

### 2. 实现

- `XlStorage` (`src/components/xl_storage/`) - StorageAPI 实现
- `ErasureServerPools` (`src/components/erasure_server_pools/`) - ObjectLayer 实现

### 3. 类型

- `object_layer_types.rs` - ObjectLayer 类型
- `storage_types.rs` - StorageAPI 类型

## 旧架构 vs 新架构

| 旧架构 | 新架构 | 说明 |
|--------|--------|------|
| S3Mount | StorageAPI | 底层存储接口 |
| S3MetadataStorage | StorageAPI (metadata ops) | 元数据操作整合到 StorageAPI |
| S3Engine | ObjectLayer | 高层对象操作 |

## 迁移步骤

1. 使用 `XlStorage` 替代 `LocalFsMount` 和 `MemoryMount`
2. 使用 `ErasureServerPools` 替代 `S3Engine`
3. 元数据操作通过 `StorageAPI` 的 `read_version/write_metadata` 完成

## 示例

```rust
let storage = Arc::new(XlStorage::new(PathBuf::from("/data")));
let object_layer = ErasureServerPools::new(storage);
```
