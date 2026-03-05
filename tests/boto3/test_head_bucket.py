#!/usr/bin/env python3
"""Test head_bucket API"""
from client_helper import create_client, get_endpoint

def test_head_bucket():
    s3 = create_client(get_endpoint())
    bucket = "test-head-bucket"

    s3.create_bucket(Bucket=bucket)
    resp = s3.head_bucket(Bucket=bucket)
    print(f"✓ head_bucket: {resp['ResponseMetadata']['HTTPStatusCode']}")

    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_head_bucket()
