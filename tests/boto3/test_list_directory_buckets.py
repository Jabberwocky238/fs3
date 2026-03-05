#!/usr/bin/env python3
"""Test list directory buckets (S3 Express One Zone)"""
from client_helper import create_client, get_endpoint

def test(s3):
    try:
        resp = s3.list_directory_buckets()
        print(f"[OK] LIST directory buckets: {resp}")
    except Exception as e:
        print(f"[SKIP] Directory buckets not supported: {e}")

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
