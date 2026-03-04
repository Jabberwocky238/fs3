#!/usr/bin/env python3
"""Test upload/download with metadata"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-metadata"
    key = "test.txt"
    s3.create_bucket(Bucket=bucket)

    s3.put_object(Bucket=bucket, Key=key, Body=b"data", Metadata={"author": "test", "version": "1"})
    print("[OK] PUT object with metadata")

    resp = s3.head_object(Bucket=bucket, Key=key)
    assert resp["Metadata"]["author"] == "test"
    print("[OK] GET object metadata")

    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
