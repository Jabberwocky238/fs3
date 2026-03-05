#!/usr/bin/env python3
"""Test bucket accelerate configuration"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-accelerate"
    s3.create_bucket(Bucket=bucket)

    s3.put_bucket_accelerate_configuration(Bucket=bucket, AccelerateConfiguration={"Status": "Enabled"})
    print("[OK] PUT accelerate")

    resp = s3.get_bucket_accelerate_configuration(Bucket=bucket)
    print(f"[OK] GET accelerate: {resp}")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
