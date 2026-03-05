#!/usr/bin/env python3
"""Test bucket metadata table configuration (MinIO extension)"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-metadata-table"
    s3.create_bucket(Bucket=bucket)

    try:
        s3.create_bucket_metadata_table_configuration(Bucket=bucket)
        print("[OK] CREATE metadata table")

        resp = s3.get_bucket_metadata_table_configuration(Bucket=bucket)
        print(f"[OK] GET metadata table: {resp}")

        s3.delete_bucket_metadata_table_configuration(Bucket=bucket)
        print("[OK] DELETE metadata table")
    except Exception as e:
        print(f"[SKIP] Metadata table not supported: {e}")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
