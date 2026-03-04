#!/usr/bin/env python3
"""Test HEAD object operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-head"
    key = "test.txt"
    s3.create_bucket(Bucket=bucket)
    s3.put_object(Bucket=bucket, Key=key, Body=b"data")

    resp = s3.head_object(Bucket=bucket, Key=key)
    assert resp["ContentLength"] == 4
    print("[OK] HEAD object")

    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
