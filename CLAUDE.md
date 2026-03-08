# FS3 Agent Guide

## Project Goal
Build a lightweight S3-compatible object storage gateway in Rust, compatible with MinIO API.

## Finding Truth

0. **ROADMAP**: `README.md` - ALL FEATURES
1. **Handler Traits**: `src/types/traits/s3_handler/*.rs` - API contracts
2. **Engine Traits**: `src/types/traits/s3_engine/*.rs` - Storage contracts
3. **MinIO Reference**: `minio/cmd/*-handlers.go` - Official implementation
4. **Request Types**: `src/types/s3/request.rs` - Input structures
5. **Response Types**: `src/types/s3/response.rs` - Output structures

## Testing

**IMPORTANT: Run tests ONE AT A TIME, not all together**

### Two-Phase Testing Strategy

**大多数boto3测试需要拆分为两个阶段：**

1. **Phase 1 (写入阶段)**: 创建资源、写入数据
2. **Phase 2 (修改/删除阶段)**: 修改或删除资源

**测试流程：**
```bash
# 1. 启动服务器
minio.exe server --address 127.0.0.1:9000 --console-address 127.0.0.1:9001 .debug/minio
fs3.exe server --address 127.0.0.1:9100 .debug/fs3
# 无需sleep等待服务器启动，服务器会瞬间启动

# 2. 执行Phase 1 - MinIO和fs3都写入数据
python test_xxx.py http://127.0.0.1:9000 1
python test_xxx.py http://127.0.0.1:9100 1

# 3. 停止服务器
# 调用claude code stop

# 4. 交换存储挂载点重启服务器（挂载点已交换）
minio.exe server --address 127.0.0.1:9000 --console-address 127.0.0.1:9001 .debug/fs3
fs3.exe server --address 127.0.0.1:9100 .debug/minio

# 5. 执行Phase 2 - 观察跨实现的存储兼容性
python test_xxx.py http://127.0.0.1:9000 2  # MinIO读取fs3的存储
python test_xxx.py http://127.0.0.1:9100 2  # fs3读取MinIO的存储
```

**目的：验证存储格式兼容性，确保fs3和MinIO可以互相读取对方的存储数据。**

出现任何测试问题，第一步，检查minio源码如何实现，第二步，修改fs3源码适配，第三步，重新测试。

当前有三个测试文件夹，aws，minio和boto3，aws和minio是rust的集成测试，使用cargo test启动。

```bash
# Run single test (RECOMMENDED)
cargo test --test minio_tests test_put_bucket_cors

# Run specific test file
cargo test --test minio_tests cors

# DO NOT run all tests at once
# cargo test --test minio_tests  # ❌ AVOID THIS

# Verify compilation (faster than build)
cargo check
```
tests/boto3是个特殊文件夹，使用`cd /d/1-code/__trash__/fs3/tests/boto3 && python test_xxx.py http://127.0.0.1:9000`来进行调用。

**你需要使用make build和make build-minio来构建minio.exe和fs3.exe，一定要使用make**，因为涉及到copy操作，cargo不会构建在根目录。

构建完成后，在项目目录下启动。你需要把它们放到CLAUDE CODE后台任务，便于管理。

## 设计准则

src\types\s3文件夹承载了整个仓库所有的类型。

任何XML请求在请求进入的时候，axum router读取到裸请求，需要被转化为具体字段放入request结构体，返回时，需要由response结构体进行序列化为xml。

request和response结构体里不允许出现类似`xml`或者`json`的字段名，因为他们应该被事先解码，而不是存储裸数据。

任何xml裸字符串都不应该进入engine层以及更下层，解码时可以考虑使用builder设计模式或者工厂函数。

当前的代码实现有很多地方违背了这一点，你需要注意和修正。

src\types\traits文件夹承载了整个仓库所有的核心trait

有必要时，对trait方法进行增加是允许的。

## CRITICAL: Development Workflow

**When implementing or modifying ANY feature, follow this strict 3-step process:**

### Step 1: Check What Is Correct
- Read MinIO source code (`minio/cmd/*-handlers.go`)
- Check S3 API documentation
- Understand expected behavior, edge cases, error codes

### Step 2: Implement Test First
- Write test in `tests/minio/*.rs`
- Test MUST fail initially (feature not implemented)
- Test MUST cover expected behavior from Step 1
- Use `cargo check` to verify test compiles

### Step 3: Implement Core
- Implement feature in `src/`
- Run `cargo check` frequently
- **CRITICAL**: DO NOT modify tests in this step
- **If test is wrong, STOP immediately and report the issue**

### Step 4: Run Tests
- Run `cargo test --test minio_tests <feature>`
- Test MUST pass
- If test fails, fix implementation (NOT the test)
- Repeat until all tests pass

### Step 5: Update Documentation
- ONLY after tests pass, update `README.md`
- Mark feature in roadmap table
- Update test coverage count

## Roadmap

See `README.md` for complete feature list with MinIO source references.

## MinIO Behavior Documentation

**Record actual MinIO filesystem changes in `docs/minio_actual/`**

For each operation, document:
1. Initial state (directory tree before operation)
2. Operation performed (boto3 command)
3. Final state (directory tree after operation)
4. File contents (xl.meta, data files)

This provides ground truth for fs3 implementation.
