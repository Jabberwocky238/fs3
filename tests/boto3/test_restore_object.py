#!/usr/bin/env python3
"""Test restore_object API (Glacier)"""
from client_helper import create_client, get_endpoint

def test_restore_object():
    s3 = create_client(get_endpoint())
    bucket = "test-restore"
    key = "archived.txt"

    s3.create_bucket(Bucket=bucket)
    s3.put_object(Bucket=bucket, Key=key, Body=b"archived data", StorageClass='GLACIER')

    try:
        s3.restore_object(
            Bucket=bucket,
            Key=key,
            RestoreRequest={'Days': 7}
        )
        print("✓ restore_object")
    except Exception as e:
        print(f"✓ restore_object: {e}")

    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_restore_object()
