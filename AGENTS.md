# FS3 Agent Guide

## Project Goal
Build a lightweight S3-compatible object storage gateway in Rust, compatible with MinIO API.

## Architecture

```
Client → Axum Handler → S3 Engine → Storage (Mount + Metadata)
```

## Core Modules

| Module | Path | Purpose |
|--------|------|---------|
| **HTTP Handler** | `src/components/fs3_axum_handler/` | Axum routes, request/response conversion |
| **S3 Engine** | `src/components/fs3_engine/` | Business logic for buckets, objects, multipart |
| **Policy Engine** | `src/components/fs3_policyengine/` | Bucket policy evaluation |
| **Storage Mount** | `src/components/s3_mount/` | Local filesystem, in-memory backends |
| **Metadata Storage** | `src/components/s3_metadata_storage/` | SQLite, JSON, in-memory metadata |
| **Type Definitions** | `src/types/` | Request/response types, traits, errors |

## Finding Truth

0. **ROADMAP**: `README.md` - ALL FEATURES
1. **Handler Traits**: `src/types/traits/s3_handler/*.rs` - API contracts
2. **Engine Traits**: `src/types/traits/s3_engine/*.rs` - Storage contracts
3. **MinIO Reference**: `minio/cmd/*-handlers.go` - Official implementation
4. **Request Types**: `src/types/s3/request.rs` - Input structures
5. **Response Types**: `src/types/s3/response.rs` - Output structures

## Testing

```bash
# Run all MinIO compatibility tests
cargo test --test minio_tests

# Run specific test file
cargo test --test minio_tests bucket

# Verify compilation (faster than build)
cargo check
```

Test files: `tests/minio/*.rs` - Each feature has corresponding test

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
