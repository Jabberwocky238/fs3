#!/usr/bin/env python3
"""Test range GET operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-range"
    key = "test.txt"
    s3.create_bucket(Bucket=bucket)
    s3.put_object(Bucket=bucket, Key=key, Body=b"0123456789")

    resp = s3.get_object(Bucket=bucket, Key=key, Range="bytes=0-4")
    assert resp["Body"].read() == b"01234"
    print("[OK] Range GET")

    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
