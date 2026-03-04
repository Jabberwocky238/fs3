#!/usr/bin/env python3
"""Test object PUT/GET/DELETE operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-objects"
    s3.create_bucket(Bucket=bucket)

    # PUT object
    s3.put_object(Bucket=bucket, Key="test.txt", Body=b"hello world")
    print("[OK] PUT object")

    # GET object
    obj = s3.get_object(Bucket=bucket, Key="test.txt")
    assert obj["Body"].read() == b"hello world"
    print("[OK] GET object")

    # DELETE object
    s3.delete_object(Bucket=bucket, Key="test.txt")
    print("[OK] DELETE object")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
