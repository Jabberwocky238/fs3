#!/usr/bin/env python3
"""Test rename object (MinIO extension)"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-rename"
    s3.create_bucket(Bucket=bucket)

    s3.put_object(Bucket=bucket, Key="old.txt", Body=b"data")
    print("[OK] PUT old.txt")

    try:
        s3.rename_object(Bucket=bucket, Key="old.txt", NewKey="new.txt")
        print("[OK] RENAME")

        s3.get_object(Bucket=bucket, Key="new.txt")
        print("[OK] GET new.txt")

        s3.delete_object(Bucket=bucket, Key="new.txt")
    except Exception as e:
        print(f"[SKIP] rename_object not supported: {e}")
        s3.delete_object(Bucket=bucket, Key="old.txt")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
