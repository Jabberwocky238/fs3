#!/usr/bin/env python3
"""Test abort multipart upload"""
import sys
from client_helper import create_client

def phase1(s3):
    """Phase 1: Create multipart upload"""
    bucket = "test-abort-multipart"
    key = "test.bin"
    s3.create_bucket(Bucket=bucket)
    print("[Phase 1] Created bucket")

    upload_id = s3.create_multipart_upload(Bucket=bucket, Key=key)["UploadId"]
    print(f"[Phase 1] Created multipart upload: {upload_id}")

def phase2(s3):
    """Phase 2: Abort multipart upload"""
    bucket = "test-abort-multipart"
    key = "test.bin"

    uploads = s3.list_multipart_uploads(Bucket=bucket)
    upload_id = uploads["Uploads"][0]["UploadId"]
    print(f"[Phase 2] Found upload: {upload_id}")

    s3.abort_multipart_upload(Bucket=bucket, Key=key, UploadId=upload_id)
    print("[Phase 2] Aborted multipart upload")

    s3.delete_bucket(Bucket=bucket)
    print("[Phase 2] Cleanup OK")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python test_abort_multipart.py <endpoint> <phase>")
        print("  phase: 1 (create) or 2 (abort)")
        sys.exit(1)

    endpoint = sys.argv[1]
    phase = sys.argv[2]

    print(f"Testing: {endpoint} - Phase {phase}")
    s3 = create_client(endpoint)

    if phase == "1":
        phase1(s3)
    elif phase == "2":
        phase2(s3)
    else:
        print("Invalid phase. Use 1 or 2")
        sys.exit(1)

    print(f"[OK] Phase {phase} completed!")
