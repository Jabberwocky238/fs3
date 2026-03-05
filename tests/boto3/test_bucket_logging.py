#!/usr/bin/env python3
"""Test bucket logging"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-logging"
    target = "test-logs"
    s3.create_bucket(Bucket=bucket)
    s3.create_bucket(Bucket=target)

    logging = {
        "LoggingEnabled": {
            "TargetBucket": target,
            "TargetPrefix": "logs/"
        }
    }
    s3.put_bucket_logging(Bucket=bucket, BucketLoggingStatus=logging)
    print("[OK] PUT logging")

    resp = s3.get_bucket_logging(Bucket=bucket)
    print(f"[OK] GET logging: {resp}")

    s3.delete_bucket(Bucket=bucket)
    s3.delete_bucket(Bucket=target)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
