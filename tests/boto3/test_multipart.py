#!/usr/bin/env python3
"""Test multipart upload operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-multipart"
    key = "large-file.bin"
    s3.create_bucket(Bucket=bucket)

    # Initiate multipart upload
    resp = s3.create_multipart_upload(Bucket=bucket, Key=key)
    upload_id = resp["UploadId"]
    print("[OK] Initiated multipart upload")

    # Upload parts
    parts = []
    for i in range(1, 3):
        part = s3.upload_part(Bucket=bucket, Key=key, PartNumber=i, UploadId=upload_id, Body=b"x" * 5242880)
        parts.append({"PartNumber": i, "ETag": part["ETag"]})
    print("[OK] Uploaded parts")

    # Complete multipart upload
    s3.complete_multipart_upload(Bucket=bucket, Key=key, UploadId=upload_id, MultipartUpload={"Parts": parts})
    print("[OK] Completed multipart upload")

    # Cleanup
    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
