#!/usr/bin/env python3
"""Test copy object operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-copy"
    s3.create_bucket(Bucket=bucket)
    s3.put_object(Bucket=bucket, Key="source.txt", Body=b"data")

    s3.copy_object(Bucket=bucket, Key="dest.txt", CopySource={"Bucket": bucket, "Key": "source.txt"})
    print("[OK] Copy object")

    s3.delete_object(Bucket=bucket, Key="source.txt")
    s3.delete_object(Bucket=bucket, Key="dest.txt")
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
