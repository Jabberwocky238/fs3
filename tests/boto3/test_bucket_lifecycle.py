#!/usr/bin/env python3
"""Test bucket lifecycle operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-lifecycle"
    s3.create_bucket(Bucket=bucket)

    lifecycle = {
        "Rules": [{
            "ID": "rule1",
            "Status": "Enabled",
            "Prefix": "logs/",
            "Expiration": {"Days": 30}
        }]
    }
    s3.put_bucket_lifecycle_configuration(Bucket=bucket, LifecycleConfiguration=lifecycle)
    print("[OK] PUT bucket lifecycle")

    resp = s3.get_bucket_lifecycle_configuration(Bucket=bucket)
    assert len(resp["Rules"]) == 1
    print("[OK] GET bucket lifecycle")

    s3.delete_bucket_lifecycle(Bucket=bucket)
    print("[OK] DELETE bucket lifecycle")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
