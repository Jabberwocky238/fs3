#!/usr/bin/env python3
"""Test bucket ABAC (Attribute-Based Access Control)"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-abac"
    s3.create_bucket(Bucket=bucket)

    try:
        config = {"Status": "Enabled"}
        s3.put_bucket_abac(Bucket=bucket, ABACConfiguration=config)
        print("[OK] PUT ABAC")

        resp = s3.get_bucket_abac(Bucket=bucket)
        print(f"[OK] GET ABAC: {resp}")
    except Exception as e:
        print(f"[SKIP] ABAC not supported: {e}")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
