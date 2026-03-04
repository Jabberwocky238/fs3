#!/usr/bin/env python3
from client_helper import create_client, get_endpoint
import sys

endpoint = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:9000"
s3 = create_client(endpoint)

bucket = "test-mp-observe"
key = "large.bin"

s3.create_bucket(Bucket=bucket)
print("[OK] Created bucket")

# Initiate
resp = s3.create_multipart_upload(Bucket=bucket, Key=key)
upload_id = resp["UploadId"]
print(f"[OK] Initiated: {upload_id}")
input("Press Enter to upload parts...")

# Upload parts
parts = []
for i in range(1, 3):
    part = s3.upload_part(Bucket=bucket, Key=key, PartNumber=i, UploadId=upload_id, Body=b"x" * 5242880)
    parts.append({"PartNumber": i, "ETag": part["ETag"]})
    print(f"[OK] Uploaded part {i}")
input("Press Enter to complete...")

# Complete
s3.complete_multipart_upload(Bucket=bucket, Key=key, UploadId=upload_id, MultipartUpload={"Parts": parts})
print("[OK] Completed")
input("Press Enter to cleanup...")

s3.delete_object(Bucket=bucket, Key=key)
s3.delete_bucket(Bucket=bucket)
print("[OK] Cleaned up")
