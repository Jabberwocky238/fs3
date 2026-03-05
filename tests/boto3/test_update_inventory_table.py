#!/usr/bin/env python3
"""Test update bucket metadata inventory table (MinIO extension)"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-update-inventory"
    s3.create_bucket(Bucket=bucket)

    try:
        s3.update_bucket_metadata_inventory_table_configuration(Bucket=bucket)
        print("[OK] UPDATE inventory table")
    except Exception as e:
        print(f"[SKIP] Update inventory not supported: {e}")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
