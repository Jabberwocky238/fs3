#!/usr/bin/env python3
"""Test presigned URL operations"""
from client_helper import create_client, get_endpoint
import requests

def test(s3):
    bucket = "test-presigned"
    key = "test.txt"
    s3.create_bucket(Bucket=bucket)
    s3.put_object(Bucket=bucket, Key=key, Body=b"data")

    url = s3.generate_presigned_url("get_object", Params={"Bucket": bucket, "Key": key}, ExpiresIn=3600)
    resp = requests.get(url)
    assert resp.status_code == 200
    print("[OK] Presigned URL")

    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
