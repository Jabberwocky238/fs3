#!/usr/bin/env python3
"""Test bucket CORS operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-cors"
    s3.create_bucket(Bucket=bucket)

    cors = {
        "CORSRules": [{
            "AllowedMethods": ["GET", "PUT"],
            "AllowedOrigins": ["*"],
            "AllowedHeaders": ["*"]
        }]
    }
    s3.put_bucket_cors(Bucket=bucket, CORSConfiguration=cors)
    print("[OK] PUT bucket CORS")

    resp = s3.get_bucket_cors(Bucket=bucket)
    assert len(resp["CORSRules"]) == 1
    print("[OK] GET bucket CORS")

    s3.delete_bucket_cors(Bucket=bucket)
    print("[OK] DELETE bucket CORS")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
