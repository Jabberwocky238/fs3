#!/usr/bin/env python3
"""Test list objects operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-list"
    s3.create_bucket(Bucket=bucket)

    # PUT multiple objects
    for i in range(5):
        s3.put_object(Bucket=bucket, Key=f"file{i}.txt", Body=b"data")

    # List objects v2
    resp = s3.list_objects_v2(Bucket=bucket)
    assert len(resp["Contents"]) == 5
    print("[OK] List objects v2")

    # Cleanup
    for obj in resp["Contents"]:
        s3.delete_object(Bucket=bucket, Key=obj["Key"])
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
