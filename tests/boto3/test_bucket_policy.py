#!/usr/bin/env python3
"""Test bucket policy operations"""
from client_helper import create_client, get_endpoint
import json

def test(s3):
    bucket = "test-policy"
    s3.create_bucket(Bucket=bucket)

    policy = {
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": "*",
            "Action": "s3:GetObject",
            "Resource": f"arn:aws:s3:::{bucket}/*"
        }]
    }
    s3.put_bucket_policy(Bucket=bucket, Policy=json.dumps(policy))
    print("[OK] PUT bucket policy")

    resp = s3.get_bucket_policy(Bucket=bucket)
    assert "Statement" in json.loads(resp["Policy"])
    print("[OK] GET bucket policy")

    s3.delete_bucket_policy(Bucket=bucket)
    print("[OK] DELETE bucket policy")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
