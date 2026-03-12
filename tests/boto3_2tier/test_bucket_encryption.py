#!/usr/bin/env python3
"""Test bucket encryption operations"""
import sys
from client_helper import create_client

def phase1(s3):
    """Phase 1: Create bucket and set encryption"""
    bucket = "test-encryption"
    s3.create_bucket(Bucket=bucket)
    print("[Phase 1] Created bucket")

    encryption = {
        "Rules": [{
            "ApplyServerSideEncryptionByDefault": {
                "SSEAlgorithm": "AES256"
            }
        }]
    }
    s3.put_bucket_encryption(Bucket=bucket, ServerSideEncryptionConfiguration=encryption)
    print("[Phase 1] PUT bucket encryption")

def phase2(s3):
    """Phase 2: Read and delete encryption"""
    bucket = "test-encryption"

    resp = s3.get_bucket_encryption(Bucket=bucket)
    assert len(resp["Rules"]) == 1
    print("[Phase 2] GET bucket encryption OK")

    s3.delete_bucket_encryption(Bucket=bucket)
    print("[Phase 2] DELETE bucket encryption")

    s3.delete_bucket(Bucket=bucket)
    print("[Phase 2] Cleanup OK")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python test_bucket_encryption.py <endpoint> <phase>")
        print("  phase: 1 (write) or 2 (read/delete)")
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
