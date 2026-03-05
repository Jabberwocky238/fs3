#!/usr/bin/env python3
"""Test bucket metrics configuration"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-metrics"
    s3.create_bucket(Bucket=bucket)

    config = {"Id": "metrics1"}
    s3.put_bucket_metrics_configuration(Bucket=bucket, Id="metrics1", MetricsConfiguration=config)
    print("[OK] PUT metrics")

    resp = s3.get_bucket_metrics_configuration(Bucket=bucket, Id="metrics1")
    print(f"[OK] GET metrics: {resp}")

    resp = s3.list_bucket_metrics_configurations(Bucket=bucket)
    print(f"[OK] LIST metrics: {resp}")

    s3.delete_bucket_metrics_configuration(Bucket=bucket, Id="metrics1")
    print("[OK] DELETE metrics")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
