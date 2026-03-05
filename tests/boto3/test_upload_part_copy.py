#!/usr/bin/env python3
"""Test upload_part_copy API"""
from client_helper import create_client, get_endpoint

def test_upload_part_copy():
    s3 = create_client(get_endpoint())
    bucket = "test-part-copy"
    src_key = "source.txt"
    dst_key = "dest.txt"

    s3.create_bucket(Bucket=bucket)
    s3.put_object(Bucket=bucket, Key=src_key, Body=b"a" * 10_000_000)

    # Start multipart upload
    resp = s3.create_multipart_upload(Bucket=bucket, Key=dst_key)
    upload_id = resp['UploadId']
    print(f"✓ create_multipart_upload: {upload_id}")

    # Upload part copy
    resp = s3.upload_part_copy(
        Bucket=bucket,
        Key=dst_key,
        CopySource={'Bucket': bucket, 'Key': src_key},
        PartNumber=1,
        UploadId=upload_id
    )
    etag = resp['CopyPartResult']['ETag']
    print(f"✓ upload_part_copy: {etag}")

    # Complete
    s3.complete_multipart_upload(
        Bucket=bucket,
        Key=dst_key,
        UploadId=upload_id,
        MultipartUpload={'Parts': [{'ETag': etag, 'PartNumber': 1}]}
    )
    print("✓ complete_multipart_upload")

    s3.delete_object(Bucket=bucket, Key=src_key)
    s3.delete_object(Bucket=bucket, Key=dst_key)
    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_upload_part_copy()
