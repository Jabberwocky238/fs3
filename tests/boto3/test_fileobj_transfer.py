#!/usr/bin/env python3
"""Test upload_fileobj and download_fileobj APIs"""
from client_helper import create_client, get_endpoint
from io import BytesIO

def test_fileobj_transfer():
    s3 = create_client(get_endpoint())
    bucket = "test-fileobj"
    key = "uploaded.txt"

    s3.create_bucket(Bucket=bucket)

    # Upload from BytesIO
    upload_data = BytesIO(b"test content from BytesIO")
    s3.upload_fileobj(upload_data, bucket, key)
    print("✓ upload_fileobj")

    # Download to BytesIO
    download_data = BytesIO()
    s3.download_fileobj(bucket, key, download_data)
    print(f"✓ download_fileobj: {len(download_data.getvalue())} bytes")

    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_fileobj_transfer()
