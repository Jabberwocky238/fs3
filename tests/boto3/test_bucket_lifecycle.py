#!/usr/bin/env python3
"""Test bucket lifecycle operations"""
import sys
from client_helper import create_client

def phase1(s3):
    """Phase 1: Create bucket and set lifecycle"""
    bucket = "test-lifecycle"
    s3.create_bucket(Bucket=bucket)
    print("[Phase 1] Created bucket")

    lifecycle = {
        "Rules": [{
            "ID": "rule1",
            "Status": "Enabled",
            "Prefix": "logs/",
            "Expiration": {"Days": 30}
        }]
    }
    s3.put_bucket_lifecycle_configuration(Bucket=bucket, LifecycleConfiguration=lifecycle)
    print("[Phase 1] PUT bucket lifecycle")

def phase2(s3):
    """Phase 2: Read and delete lifecycle"""
    bucket = "test-lifecycle"

    resp = s3.get_bucket_lifecycle_configuration(Bucket=bucket)
    assert len(resp["Rules"]) == 1
    print("[Phase 2] GET bucket lifecycle OK")

    s3.delete_bucket_lifecycle(Bucket=bucket)
    print("[Phase 2] DELETE bucket lifecycle")

    s3.delete_bucket(Bucket=bucket)
    print("[Phase 2] Cleanup OK")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python test_bucket_lifecycle.py <endpoint> <phase>")
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
