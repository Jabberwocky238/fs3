#!/usr/bin/env python3
"""Test get_object_attributes API"""
from client_helper import create_client, get_endpoint

def test_object_attributes():
    s3 = create_client(get_endpoint())
    bucket = "test-obj-attrs"
    key = "test.txt"

    s3.create_bucket(Bucket=bucket)
    s3.put_object(Bucket=bucket, Key=key, Body=b"test data")

    # Get object attributes
    resp = s3.get_object_attributes(
        Bucket=bucket,
        Key=key,
        ObjectAttributes=['ETag', 'StorageClass', 'ObjectSize']
    )
    print(f"✓ get_object_attributes: Size={resp.get('ObjectSize')}, ETag={resp.get('ETag')}")

    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_object_attributes()
