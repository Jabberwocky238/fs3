#!/usr/bin/env python3
"""Two-phase test example: object operations"""
import sys
from client_helper import create_client, get_endpoint

def phase1(s3):
    """Phase 1: Write data"""
    bucket = "phase-test"
    s3.create_bucket(Bucket=bucket)
    print("[Phase 1] Created bucket")

    s3.put_object(Bucket=bucket, Key="test.txt", Body=b"hello world")
    print("[Phase 1] PUT object")

def phase2(s3):
    """Phase 2: Read and modify"""
    bucket = "phase-test"

    obj = s3.get_object(Bucket=bucket, Key="test.txt")
    data = obj["Body"].read()
    assert data == b"hello world"
    print("[Phase 2] GET object OK")

    s3.delete_object(Bucket=bucket, Key="test.txt")
    s3.delete_bucket(Bucket=bucket)
    print("[Phase 2] Cleanup OK")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python test_two_phase_example.py <endpoint> <phase>")
        print("  phase: 1 (write) or 2 (read/modify)")
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
