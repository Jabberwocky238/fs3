#!/usr/bin/env python3
"""Test object ACL operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-obj-acl"
    key = "test.txt"
    s3.create_bucket(Bucket=bucket)
    s3.put_object(Bucket=bucket, Key=key, Body=b"data")

    s3.put_object_acl(Bucket=bucket, Key=key, ACL="public-read")
    print("[OK] PUT object ACL")

    resp = s3.get_object_acl(Bucket=bucket, Key=key)
    assert "Grants" in resp
    print("[OK] GET object ACL")

    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
