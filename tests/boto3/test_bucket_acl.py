#!/usr/bin/env python3
"""Test bucket ACL operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-acl"
    s3.create_bucket(Bucket=bucket)

    s3.put_bucket_acl(Bucket=bucket, ACL="public-read")
    print("[OK] PUT bucket ACL")

    resp = s3.get_bucket_acl(Bucket=bucket)
    assert "Grants" in resp
    print("[OK] GET bucket ACL")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
