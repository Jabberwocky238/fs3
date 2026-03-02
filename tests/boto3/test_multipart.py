"""Tests for multipart upload operations."""
import hashlib


def test_multipart_upload(s3, bucket):
    key = "multipart.bin"
    resp = s3.create_multipart_upload(Bucket=bucket, Key=key)
    upload_id = resp["UploadId"]

    part1 = b"a" * 1024
    part2 = b"b" * 1024
    r1 = s3.upload_part(Bucket=bucket, Key=key, UploadId=upload_id, PartNumber=1, Body=part1)
    r2 = s3.upload_part(Bucket=bucket, Key=key, UploadId=upload_id, PartNumber=2, Body=part2)

    complete = s3.complete_multipart_upload(
        Bucket=bucket, Key=key, UploadId=upload_id,
        MultipartUpload={"Parts": [
            {"PartNumber": 1, "ETag": r1["ETag"]},
            {"PartNumber": 2, "ETag": r2["ETag"]},
        ]},
    )

    # Verify multipart ETag format: "md5hash-2"
    etag = complete["ETag"].strip('"')
    assert etag.endswith("-2")

    # Verify content
    resp = s3.get_object(Bucket=bucket, Key=key)
    assert resp["Body"].read() == part1 + part2
