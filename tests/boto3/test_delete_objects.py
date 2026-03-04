#!/usr/bin/env python3
"""Test delete multiple objects"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-delete-multi"
    s3.create_bucket(Bucket=bucket)

    for i in range(3):
        s3.put_object(Bucket=bucket, Key=f"file{i}.txt", Body=b"data")

    s3.delete_objects(Bucket=bucket, Delete={"Objects": [{"Key": f"file{i}.txt"} for i in range(3)]})
    print("[OK] Delete multiple objects")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
