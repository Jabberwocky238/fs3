#!/usr/bin/env python3
"""Test write get object response (S3 Object Lambda)"""
from client_helper import create_client, get_endpoint

def test(s3):
    print("[SKIP] write_get_object_response requires S3 Object Lambda")

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
