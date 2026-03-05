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

构建完成后，在项目目录下启动。你需要把它们放到后台任务，便于管理。

```bash
fs3.exe server --address 127.0.0.1:9100 .debug/fs3

minio.exe  server --address 127.0.0.1:9000 --console-address 127.0.0.1:9001 .debug/minio
```

你需要把它们放到后台任务，便于管理。无需等待服务器启动，二者都是瞬间启动的。

然后使用boto3文件夹里的测试py脚本，分别调用两个api，每一轮小测试，交叉调用两个api

tests/boto3是个特殊文件夹，使用`cd /d/1-code/__trash__/fs3/tests/boto3 && python test_xxx.py http://127.0.0.1:9000`来进行调用。

！！观察文件夹目录树变化，
！！观察文件夹目录树变化，
！！观察文件夹目录树变化，

如果fs3有行为不一致的地方，则需要修改，如果minio有变化而fs3没有，需要修改。

测试结果OK也不是测试OK，你一定要**观察目录树变化**。

你需要开始循环测试，每次测试一对，交叉测试minio和fs3，然后根据fs3的行为不一致，来修改aws和minio测试，以及fs3核心。

tests/boto3是个特殊文件夹，使用`cd /d/1-code/__trash__/fs3/tests/boto3 && python test_xxx.py http://127.0.0.1:9000`来进行调用。

！！观察文件夹目录树变化，
！！观察文件夹目录树变化，
！！观察文件夹目录树变化，

当你遇到棘手的问题时，找不到解决方案，或者没有好的解决方案，此时观察submodule minio，minio的源码就在项目目录下，你需要学习它的代码，然后实现。

每次测试出现问题，结束之后，需要关闭两个进程，重新编译fs3，但不需要编译minio，删除.debug文件夹，然后重新开启两个进程，保证测试之间不会干扰。

！！观察文件夹目录树变化，
！！观察文件夹目录树变化，
！！观察文件夹目录树变化，

如果测试没有出问题，就可以继续，不需要关闭进程。

！！观察文件夹目录树变化，
！！观察文件夹目录树变化，
！！观察文件夹目录树变化，

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
