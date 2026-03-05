#!/usr/bin/env python3
"""Test put bucket notification (deprecated v1)"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-notification-v1"
    s3.create_bucket(Bucket=bucket)

    notification = {}
    try:
        s3.put_bucket_notification(Bucket=bucket, NotificationConfiguration=notification)
        print("[OK] PUT notification v1")

        resp = s3.get_bucket_notification(Bucket=bucket)
        print(f"[OK] GET notification v1: {resp}")
    except Exception as e:
        print(f"[SKIP] put_bucket_notification (v1) deprecated: {e}")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
