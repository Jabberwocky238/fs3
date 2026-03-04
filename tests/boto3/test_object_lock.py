#!/usr/bin/env python3
"""Test object lock operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-object-lock"
    s3.create_bucket(Bucket=bucket, ObjectLockEnabledForBucket=True)

    config = {
        "ObjectLockEnabled": "Enabled",
        "Rule": {"DefaultRetention": {"Mode": "GOVERNANCE", "Days": 1}}
    }
    s3.put_object_lock_configuration(Bucket=bucket, ObjectLockConfiguration=config)
    print("[OK] PUT object lock configuration")

    resp = s3.get_object_lock_configuration(Bucket=bucket)
    assert resp["ObjectLockConfiguration"]["ObjectLockEnabled"] == "Enabled"
    print("[OK] GET object lock configuration")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
