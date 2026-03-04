#!/usr/bin/env python3
"""Test legal hold operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-legal-hold"
    key = "test.txt"
    s3.create_bucket(Bucket=bucket, ObjectLockEnabledForBucket=True)
    s3.put_object(Bucket=bucket, Key=key, Body=b"data")

    s3.put_object_legal_hold(Bucket=bucket, Key=key, LegalHold={"Status": "ON"})
    print("[OK] PUT object legal hold")

    resp = s3.get_object_legal_hold(Bucket=bucket, Key=key)
    assert resp["LegalHold"]["Status"] == "ON"
    print("[OK] GET object legal hold")

    s3.put_object_legal_hold(Bucket=bucket, Key=key, LegalHold={"Status": "OFF"})
    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
