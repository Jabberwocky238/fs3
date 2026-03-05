#!/usr/bin/env python3
"""Test copy high-level API"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-copy-api"
    s3.create_bucket(Bucket=bucket)

    s3.put_object(Bucket=bucket, Key="source.txt", Body=b"hello")
    print("[OK] PUT source")

    s3.copy({"Bucket": bucket, "Key": "source.txt"}, bucket, "dest.txt")
    print("[OK] COPY")

    obj = s3.get_object(Bucket=bucket, Key="dest.txt")
    assert obj['Body'].read() == b"hello"

    s3.delete_object(Bucket=bucket, Key="source.txt")
    s3.delete_object(Bucket=bucket, Key="dest.txt")
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
