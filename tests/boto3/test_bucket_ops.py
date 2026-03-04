#!/usr/bin/env python3
"""Test bucket operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-bucket"
    s3.create_bucket(Bucket=bucket)
    print(f"[OK] Created: {bucket}")

    buckets = [b["Name"] for b in s3.list_buckets()["Buckets"]]
    assert bucket in buckets
    print(f"[OK] Listed: {buckets}")

    s3.delete_bucket(Bucket=bucket)
    print(f"[OK] Deleted: {bucket}")

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
