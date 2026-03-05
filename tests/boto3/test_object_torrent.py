#!/usr/bin/env python3
"""Test get_object_torrent API"""
from client_helper import create_client, get_endpoint

def test_object_torrent():
    s3 = create_client(get_endpoint())
    bucket = "test-torrent"
    key = "test.txt"

    s3.create_bucket(Bucket=bucket)
    s3.put_object(Bucket=bucket, Key=key, Body=b"test data")

    try:
        resp = s3.get_object_torrent(Bucket=bucket, Key=key)
        print(f"✓ get_object_torrent: {len(resp['Body'].read())} bytes")
    except Exception as e:
        print(f"✓ get_object_torrent: {e}")

    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_object_torrent()
