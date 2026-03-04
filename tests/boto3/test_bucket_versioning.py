#!/usr/bin/env python3
"""Test bucket versioning operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-versioning"
    s3.create_bucket(Bucket=bucket)

    s3.put_bucket_versioning(Bucket=bucket, VersioningConfiguration={"Status": "Enabled"})
    print("[OK] PUT bucket versioning")

    resp = s3.get_bucket_versioning(Bucket=bucket)
    assert resp.get("Status") == "Enabled"
    print("[OK] GET bucket versioning")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
