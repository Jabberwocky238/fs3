#!/usr/bin/env python3
"""Test list objects v1"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-list-v1"
    s3.create_bucket(Bucket=bucket)

    for i in range(5):
        s3.put_object(Bucket=bucket, Key=f"file{i}.txt", Body=b"data")

    resp = s3.list_objects(Bucket=bucket)
    assert len(resp.get('Contents', [])) == 5
    print(f"[OK] LIST v1: {len(resp['Contents'])} objects")

    for obj in resp['Contents']:
        s3.delete_object(Bucket=bucket, Key=obj['Key'])
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
