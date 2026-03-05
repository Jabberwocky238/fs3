# MinIO Inline Data Behavior

## Observation Date
2026-03-05

## Small File Threshold
MinIO uses `smallFileThreshold = 128 * 1024` (128 KB) for inline data optimization.

## Directory Structure Comparison

### MinIO (with inline-data)
```
persist-bucket/
└── test.txt/
    └── xl.meta (459 bytes, contains "hello world" data)
```

### fs3 (without inline-data)
```
persist-bucket/
└── test.txt/
    ├── xl.meta (160 bytes, JSON metadata only)
    └── 0073eddb-4e89-42fd-bfb9-fe144c404575 (11 bytes, data file)
```

## Implementation Details

MinIO stores small files (<128KB) directly in xl.meta using MessagePack format with:
- `x-minio-internal-inline-data: true` in MetaSys
- Binary data embedded in xl.meta

## Impact on fs3

**Current Status**: fs3 always creates separate data files
**Impact**:
- More disk I/O for small files
- More inodes used
- Slightly slower for small file operations

**Priority**: Medium (performance optimization, not functional issue)

## Implementation Requirements

To implement inline-data in fs3:
1. Replace JSON xl.meta with MessagePack format
2. Add inline data storage in xl.meta
3. Modify read/write logic to handle inline vs external data
4. Add 128KB threshold check in put_object

## Reference
- MinIO source: `minio/cmd/xl-storage.go:60`
- Format: `minio/cmd/xl-storage-format-v2.go`
