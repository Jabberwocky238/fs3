#!/usr/bin/env python3
"""Test bucket inventory configuration"""
import sys
from client_helper import create_client

def phase1(s3):
    """Phase 1: Create bucket and set inventory"""
    bucket = "test-inventory"
    s3.create_bucket(Bucket=bucket)
    print("[Phase 1] Created bucket")

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
    print("[Phase 1] PUT inventory")

def phase2(s3):
    """Phase 2: Read and delete inventory"""
    bucket = "test-inventory"

    resp = s3.get_bucket_inventory_configuration(Bucket=bucket, Id="inventory1")
    print(f"[Phase 2] GET inventory OK")

    resp = s3.list_bucket_inventory_configurations(Bucket=bucket)
    print(f"[Phase 2] LIST inventory OK")

    s3.delete_bucket_inventory_configuration(Bucket=bucket, Id="inventory1")
    print("[Phase 2] DELETE inventory")

    s3.delete_bucket(Bucket=bucket)
    print("[Phase 2] Cleanup OK")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python test_bucket_inventory.py <endpoint> <phase>")
        print("  phase: 1 (write) or 2 (read/delete)")
        sys.exit(1)

    endpoint = sys.argv[1]
    phase = sys.argv[2]

    print(f"Testing: {endpoint} - Phase {phase}")
    s3 = create_client(endpoint)

    if phase == "1":
        phase1(s3)
    elif phase == "2":
        phase2(s3)
    else:
        print("Invalid phase. Use 1 or 2")
        sys.exit(1)

    print(f"[OK] Phase {phase} completed!")
