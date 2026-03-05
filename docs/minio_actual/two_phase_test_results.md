# Two-Phase Test Results

## Test: test_two_phase_example.py

### Phase 1 Results

**MinIO Storage (.debug/minio):**
```
persist-bucket/
└── test.txt/
    └── xl.meta (XL2 binary format with inline data)
```

**fs3 Storage (.debug/fs3):**
```
persist-bucket/
└── test.txt/
    ├── xl.meta (JSON format - WRONG!)
    └── 9b46e5de-49bb-4da9-9e04-466e840bfc7b (separate data file)
```

### Phase 2 Results (Swapped Mount Points)

**MinIO reading fs3 storage:** ❌ FAILED
- Error: `InternalError: file is corrupted`
- Reason: Cannot parse JSON xl.meta, expects XL2 binary

**fs3 reading MinIO storage:** ❌ FAILED
- Error: `InternalError: file is corrupted`
- Reason: Cannot parse XL2 binary xl.meta, expects JSON

## Critical Issues Found

### 1. xl.meta Format Mismatch
- **MinIO**: Uses XL2 binary format (msgpack encoding)
  - Header: `XL2 ` (4 bytes: 'X', 'L', '2', ' ')
  - Version: Major=1, Minor=3 (4 bytes little-endian)
  - Payload: msgpack encoded data

- **fs3**: Uses JSON format
  - Plain JSON: `{"versions":[...]}`
  - No XL2 header
  - No msgpack encoding

### 2. Inline Data Support
- **MinIO**: Small objects (11 bytes) stored inline in xl.meta
  - Flag: `xlFlagInlineData` set
  - Metadata: `x-minio-internal-inline-data: true`
  - Data appended at end of xl.meta

- **fs3**: Always creates separate data file
  - No inline optimization
  - Creates UUID-named file even for tiny objects

### 3. xl.meta Structure
**MinIO xl.meta (binary):**
- Uses msgpack for `xlMetaV2Object` struct
- Contains: VersionID, DataDir, ErasureAlgo, Size, ModTime, MetaSys, MetaUser
- Inline data appended after msgpack payload

**fs3 xl.meta (JSON):**
```json
{
  "versions": [{
    "version_id": "uuid",
    "data_dir": "uuid",
    "size": 11,
    "mod_time": 0,
    "user_metadata": {}
  }]
}
```

## Required Fixes

1. Implement XL2 binary format writer/reader
2. Add inline data support for small objects
3. Use msgpack encoding instead of JSON
4. Match MinIO's xlMetaV2Object structure
