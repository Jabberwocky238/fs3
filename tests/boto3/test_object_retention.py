#!/usr/bin/env python3
"""Test object retention operations"""
from client_helper import create_client, get_endpoint
from datetime import datetime, timedelta

def test(s3):
    bucket = "test-retention"
    key = "test.txt"
    s3.create_bucket(Bucket=bucket, ObjectLockEnabledForBucket=True)
    s3.put_object(Bucket=bucket, Key=key, Body=b"data")

    retention = {
        "Mode": "GOVERNANCE",
        "RetainUntilDate": datetime.utcnow() + timedelta(days=1)
    }
    s3.put_object_retention(Bucket=bucket, Key=key, Retention=retention)
    print("[OK] PUT object retention")

    resp = s3.get_object_retention(Bucket=bucket, Key=key)
    assert resp["Retention"]["Mode"] == "GOVERNANCE"
    print("[OK] GET object retention")

    s3.delete_object(Bucket=bucket, Key=key, BypassGovernanceRetention=True)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
