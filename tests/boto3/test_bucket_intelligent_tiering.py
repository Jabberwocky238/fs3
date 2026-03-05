#!/usr/bin/env python3
"""Test bucket intelligent tiering configuration"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-tiering"
    s3.create_bucket(Bucket=bucket)

    config = {
        "Id": "tiering1",
        "Status": "Enabled",
        "Tierings": [
            {"Days": 90, "AccessTier": "ARCHIVE_ACCESS"},
            {"Days": 180, "AccessTier": "DEEP_ARCHIVE_ACCESS"}
        ]
    }
    s3.put_bucket_intelligent_tiering_configuration(Bucket=bucket, Id="tiering1", IntelligentTieringConfiguration=config)
    print("[OK] PUT intelligent tiering")

    resp = s3.get_bucket_intelligent_tiering_configuration(Bucket=bucket, Id="tiering1")
    print(f"[OK] GET intelligent tiering: {resp}")

    resp = s3.list_bucket_intelligent_tiering_configurations(Bucket=bucket)
    print(f"[OK] LIST intelligent tiering: {resp}")

    s3.delete_bucket_intelligent_tiering_configuration(Bucket=bucket, Id="tiering1")
    print("[OK] DELETE intelligent tiering")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
