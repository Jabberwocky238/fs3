#!/usr/bin/env python3
"""Test abort multipart upload"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-abort-multipart"
    key = "test.bin"
    s3.create_bucket(Bucket=bucket)

    upload_id = s3.create_multipart_upload(Bucket=bucket, Key=key)["UploadId"]
    print("[OK] Created multipart upload")

    s3.abort_multipart_upload(Bucket=bucket, Key=key, UploadId=upload_id)
    print("[OK] Aborted multipart upload")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
