#!/usr/bin/env python3
"""Test bucket location"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-location"
    s3.create_bucket(Bucket=bucket)
    print(f"[OK] Created: {bucket}")

    resp = s3.get_bucket_location(Bucket=bucket)
    print(f"[OK] Location: {resp}")

    s3.delete_bucket(Bucket=bucket)
    print(f"[OK] Deleted: {bucket}")

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
