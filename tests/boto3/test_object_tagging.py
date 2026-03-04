#!/usr/bin/env python3
"""Test object tagging operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-obj-tagging"
    key = "test.txt"
    s3.create_bucket(Bucket=bucket)
    s3.put_object(Bucket=bucket, Key=key, Body=b"data")

    tags = {"TagSet": [{"Key": "type", "Value": "document"}]}
    s3.put_object_tagging(Bucket=bucket, Key=key, Tagging=tags)
    print("[OK] PUT object tagging")

    resp = s3.get_object_tagging(Bucket=bucket, Key=key)
    assert len(resp["TagSet"]) == 1
    print("[OK] GET object tagging")

    s3.delete_object_tagging(Bucket=bucket, Key=key)
    print("[OK] DELETE object tagging")

    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
