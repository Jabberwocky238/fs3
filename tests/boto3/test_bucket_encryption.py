#!/usr/bin/env python3
"""Test bucket encryption operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-encryption"
    s3.create_bucket(Bucket=bucket)

    encryption = {
        "Rules": [{
            "ApplyServerSideEncryptionByDefault": {
                "SSEAlgorithm": "AES256"
            }
        }]
    }
    s3.put_bucket_encryption(Bucket=bucket, ServerSideEncryptionConfiguration=encryption)
    print("[OK] PUT bucket encryption")

    resp = s3.get_bucket_encryption(Bucket=bucket)
    assert len(resp["Rules"]) == 1
    print("[OK] GET bucket encryption")

    s3.delete_bucket_encryption(Bucket=bucket)
    print("[OK] DELETE bucket encryption")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
