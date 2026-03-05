#!/usr/bin/env python3
"""Test object PUT/GET/DELETE operations"""
from client_helper import create_client, get_endpoint
import sys

def make1GB():
    """Generate 1GB of data"""
    return b"0" * (1 * 1024 * 1024 * 1024)

def phase1(s3):
    bucket = "test-objects"
    s3.create_bucket(Bucket=bucket)
    data = b'x' * (50 * 1024 * 1024)
    s3.put_object(Bucket=bucket, Key="test.txt", Body=data)
    print("[OK] Phase 1: PUT 50MB object")

def phase2(s3):
    bucket = "test-objects"
    obj = s3.get_object(Bucket=bucket, Key="test.txt")
    data = obj["Body"].read()
    assert len(data) == 50 * 1024 * 1024
    print("[OK] Phase 2: GET 50MB object")

    s3.delete_object(Bucket=bucket, Key="test.txt")
    print("[OK] Phase 2: DELETE object")

if __name__ == "__main__":
    endpoint = get_endpoint()
    phase = int(sys.argv[2]) if len(sys.argv) > 2 else 0
    print(f"Testing: {endpoint} Phase: {phase}")

    s3 = create_client(endpoint)
    if phase == 1:
        phase1(s3)
    elif phase == 2:
        phase2(s3)
    else:
        phase1(s3)
        phase2(s3)

    print("[OK] All passed!")
