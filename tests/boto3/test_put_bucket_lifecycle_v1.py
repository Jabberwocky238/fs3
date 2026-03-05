#!/usr/bin/env python3
"""Test put bucket lifecycle (deprecated v1)"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-put-lifecycle-v1"
    s3.create_bucket(Bucket=bucket)

    lifecycle = {
        "Rules": [{
            "Id": "rule1",
            "Status": "Enabled",
            "Prefix": "temp/",
            "Expiration": {"Days": 7}
        }]
    }
    s3.put_bucket_lifecycle(Bucket=bucket, LifecycleConfiguration=lifecycle)
    print("[OK] PUT lifecycle v1")

    s3.delete_bucket_lifecycle(Bucket=bucket)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
