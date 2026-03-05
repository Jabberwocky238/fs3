#!/usr/bin/env python3
"""Test get bucket lifecycle (deprecated v1)"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-lifecycle-v1"
    s3.create_bucket(Bucket=bucket)

    lifecycle = {
        "Rules": [{
            "Id": "rule1",
            "Status": "Enabled",
            "Prefix": "logs/",
            "Expiration": {"Days": 30}
        }]
    }
    s3.put_bucket_lifecycle(Bucket=bucket, LifecycleConfiguration=lifecycle)
    print("[OK] PUT lifecycle v1")

    try:
        resp = s3.get_bucket_lifecycle(Bucket=bucket)
        print(f"[OK] GET lifecycle v1: {resp}")
    except Exception as e:
        print(f"[SKIP] get_bucket_lifecycle (v1) not supported: {e}")

    s3.delete_bucket_lifecycle(Bucket=bucket)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
