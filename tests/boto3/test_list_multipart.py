#!/usr/bin/env python3
"""Test list multipart uploads"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-list-multipart"
    key = "test.bin"
    s3.create_bucket(Bucket=bucket)

    upload_id = s3.create_multipart_upload(Bucket=bucket, Key=key)["UploadId"]

    resp = s3.list_multipart_uploads(Bucket=bucket)
    assert len(resp.get("Uploads", [])) >= 1
    print("[OK] List multipart uploads")

    s3.abort_multipart_upload(Bucket=bucket, Key=key, UploadId=upload_id)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
