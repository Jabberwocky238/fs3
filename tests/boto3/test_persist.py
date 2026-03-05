#!/usr/bin/env python3
"""Test object persistence - don't delete"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "persist-bucket"
    s3.create_bucket(Bucket=bucket)
    print(f"[OK] Created: {bucket}")

    s3.put_object(Bucket=bucket, Key="test.txt", Body=b"hello world")
    print("[OK] PUT object")

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] Done - check directory tree!")
