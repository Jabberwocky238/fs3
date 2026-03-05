#!/usr/bin/env python3
"""Test bucket inventory configuration"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-inventory"
    s3.create_bucket(Bucket=bucket)

    config = {
        "Id": "inventory1",
        "IsEnabled": True,
        "Destination": {
            "S3BucketDestination": {
                "Bucket": f"arn:aws:s3:::{bucket}",
                "Format": "CSV"
            }
        },
        "Schedule": {"Frequency": "Daily"},
        "IncludedObjectVersions": "Current"
    }
    s3.put_bucket_inventory_configuration(Bucket=bucket, Id="inventory1", InventoryConfiguration=config)
    print("[OK] PUT inventory")

    resp = s3.get_bucket_inventory_configuration(Bucket=bucket, Id="inventory1")
    print(f"[OK] GET inventory: {resp}")

    resp = s3.list_bucket_inventory_configurations(Bucket=bucket)
    print(f"[OK] LIST inventory: {resp}")

    s3.delete_bucket_inventory_configuration(Bucket=bucket, Id="inventory1")
    print("[OK] DELETE inventory")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
