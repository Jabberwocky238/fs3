#!/usr/bin/env python3
"""Standalone S3 test - Usage: python test.py [endpoint]"""
import sys
import boto3
from botocore.config import Config

def client(endpoint):
    return boto3.client(
        "s3",
        endpoint_url=endpoint,
        aws_access_key_id="minioadmin",
        aws_secret_access_key="minioadmin",
        config=Config(signature_version="s3v4", s3={"addressing_style": "path"}),
        region_name="us-east-1",
    )

def test(s3):
    bucket = "test-bucket"
    s3.create_bucket(Bucket=bucket)
    print(f"[OK] Created: {bucket}")

    buckets = [b["Name"] for b in s3.list_buckets()["Buckets"]]
    assert bucket in buckets
    print(f"[OK] Listed: {buckets}")

    s3.delete_bucket(Bucket=bucket)
    print(f"[OK] Deleted: {bucket}")

if __name__ == "__main__":
    endpoint = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:9000"
    print(f"Testing: {endpoint}")
    test(client(endpoint))
    print("[OK] All passed!")
