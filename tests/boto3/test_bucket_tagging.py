#!/usr/bin/env python3
"""Test bucket tagging operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-tagging"
    s3.create_bucket(Bucket=bucket)

    tags = {"TagSet": [{"Key": "env", "Value": "test"}, {"Key": "team", "Value": "dev"}]}
    s3.put_bucket_tagging(Bucket=bucket, Tagging=tags)
    print("[OK] PUT bucket tagging")

    resp = s3.get_bucket_tagging(Bucket=bucket)
    assert len(resp["TagSet"]) == 2
    print("[OK] GET bucket tagging")

    s3.delete_bucket_tagging(Bucket=bucket)
    print("[OK] DELETE bucket tagging")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
