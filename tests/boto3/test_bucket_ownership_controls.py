#!/usr/bin/env python3
"""Test bucket ownership controls"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-ownership"
    s3.create_bucket(Bucket=bucket)

    config = {
        "Rules": [
            {"ObjectOwnership": "BucketOwnerEnforced"}
        ]
    }
    s3.put_bucket_ownership_controls(Bucket=bucket, OwnershipControls=config)
    print("[OK] PUT ownership controls")

    resp = s3.get_bucket_ownership_controls(Bucket=bucket)
    print(f"[OK] GET ownership controls: {resp}")

    s3.delete_bucket_ownership_controls(Bucket=bucket)
    print("[OK] DELETE ownership controls")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
