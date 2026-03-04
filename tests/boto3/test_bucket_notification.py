#!/usr/bin/env python3
"""Test bucket notification operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-notification"
    s3.create_bucket(Bucket=bucket)

    notification = {"QueueConfigurations": []}
    s3.put_bucket_notification_configuration(Bucket=bucket, NotificationConfiguration=notification)
    print("[OK] PUT bucket notification")

    resp = s3.get_bucket_notification_configuration(Bucket=bucket)
    assert "QueueConfigurations" in resp
    print("[OK] GET bucket notification")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
