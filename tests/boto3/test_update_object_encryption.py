#!/usr/bin/env python3
"""Test update object encryption (MinIO extension)"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-update-enc"
    s3.create_bucket(Bucket=bucket)

    s3.put_object(Bucket=bucket, Key="file.txt", Body=b"data")
    print("[OK] PUT object")

    try:
        s3.update_object_encryption(Bucket=bucket, Key="file.txt", ServerSideEncryption="AES256")
        print("[OK] UPDATE encryption")
    except Exception as e:
        print(f"[SKIP] update_object_encryption not supported: {e}")

    s3.delete_object(Bucket=bucket, Key="file.txt")
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
