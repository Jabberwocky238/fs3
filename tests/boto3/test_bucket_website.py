#!/usr/bin/env python3
"""Test bucket website operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-website"
    s3.create_bucket(Bucket=bucket)

    website = {
        "IndexDocument": {"Suffix": "index.html"},
        "ErrorDocument": {"Key": "error.html"}
    }
    s3.put_bucket_website(Bucket=bucket, WebsiteConfiguration=website)
    print("[OK] PUT bucket website")

    resp = s3.get_bucket_website(Bucket=bucket)
    assert resp["IndexDocument"]["Suffix"] == "index.html"
    print("[OK] GET bucket website")

    s3.delete_bucket_website(Bucket=bucket)
    print("[OK] DELETE bucket website")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
