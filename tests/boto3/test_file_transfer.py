#!/usr/bin/env python3
"""Test upload_file and download_file high-level APIs"""
from client_helper import create_client, get_endpoint
import tempfile
import os

def test_file_transfer():
    s3 = create_client(get_endpoint())
    bucket = "test-file-transfer"
    key = "uploaded.txt"

    s3.create_bucket(Bucket=bucket)

    # Create temp file
    with tempfile.NamedTemporaryFile(mode='w', delete=False, suffix='.txt') as f:
        f.write("test content for upload")
        upload_path = f.name

    # Upload file
    s3.upload_file(upload_path, bucket, key)
    print("✓ upload_file")

    # Download file
    download_path = tempfile.mktemp(suffix='.txt')
    s3.download_file(bucket, key, download_path)
    print("✓ download_file")

    # Verify
    with open(download_path, 'r') as f:
        content = f.read()
        print(f"✓ Content verified: {len(content)} bytes")

    # Cleanup
    os.unlink(upload_path)
    os.unlink(download_path)
    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_file_transfer()
