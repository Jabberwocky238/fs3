#!/usr/bin/env python3
"""Test bucket analytics configuration"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-analytics"
    s3.create_bucket(Bucket=bucket)

    config = {
        "Id": "analytics1",
        "StorageClassAnalysis": {}
    }
    s3.put_bucket_analytics_configuration(Bucket=bucket, Id="analytics1", AnalyticsConfiguration=config)
    print("[OK] PUT analytics")

    resp = s3.get_bucket_analytics_configuration(Bucket=bucket, Id="analytics1")
    print(f"[OK] GET analytics: {resp}")

    resp = s3.list_bucket_analytics_configurations(Bucket=bucket)
    print(f"[OK] LIST analytics: {resp}")

    s3.delete_bucket_analytics_configuration(Bucket=bucket, Id="analytics1")
    print("[OK] DELETE analytics")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
