#!/usr/bin/env python3
"""Test list_parts API"""
from client_helper import create_client, get_endpoint

def test_list_parts():
    s3 = create_client(get_endpoint())
    bucket = "test-list-parts"
    key = "multipart.txt"

    s3.create_bucket(Bucket=bucket)

    # Start multipart
    resp = s3.create_multipart_upload(Bucket=bucket, Key=key)
    upload_id = resp['UploadId']

    # Upload parts
    s3.upload_part(Bucket=bucket, Key=key, PartNumber=1, UploadId=upload_id, Body=b"part1")
    s3.upload_part(Bucket=bucket, Key=key, PartNumber=2, UploadId=upload_id, Body=b"part2")

    # List parts
    resp = s3.list_parts(Bucket=bucket, Key=key, UploadId=upload_id)
    print(f"✓ list_parts: {len(resp['Parts'])} parts")

    s3.abort_multipart_upload(Bucket=bucket, Key=key, UploadId=upload_id)
    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_list_parts()
