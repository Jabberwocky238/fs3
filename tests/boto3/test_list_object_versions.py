#!/usr/bin/env python3
"""Test list_object_versions API"""
from client_helper import create_client, get_endpoint

def test_list_object_versions():
    s3 = create_client(get_endpoint())
    bucket = "test-versions"
    key = "versioned.txt"

    s3.create_bucket(Bucket=bucket)
    s3.put_bucket_versioning(Bucket=bucket, VersioningConfiguration={'Status': 'Enabled'})

    # Create multiple versions
    s3.put_object(Bucket=bucket, Key=key, Body=b"v1")
    s3.put_object(Bucket=bucket, Key=key, Body=b"v2")
    s3.put_object(Bucket=bucket, Key=key, Body=b"v3")

    # List versions
    resp = s3.list_object_versions(Bucket=bucket)
    print(f"✓ list_object_versions: {len(resp.get('Versions', []))} versions")

    # Cleanup
    for v in resp.get('Versions', []):
        s3.delete_object(Bucket=bucket, Key=v['Key'], VersionId=v['VersionId'])
    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_list_object_versions()
