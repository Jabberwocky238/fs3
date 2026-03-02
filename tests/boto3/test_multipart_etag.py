"""Tests for multipart upload ETag correctness."""
import hashlib


def test_multipart_etag_correctness(s3, bucket):
    """Verify multipart ETag matches S3 spec: MD5(MD5(p1)||MD5(p2)||...)-N"""
    key = "etag-check.bin"
    resp = s3.create_multipart_upload(Bucket=bucket, Key=key)
    upload_id = resp["UploadId"]

    parts_data = [b"x" * 512, b"y" * 512, b"z" * 512]
    parts = []
    md5_bins = []

    for i, data in enumerate(parts_data, 1):
        r = s3.upload_part(Bucket=bucket, Key=key, UploadId=upload_id, PartNumber=i, Body=data)
        parts.append({"PartNumber": i, "ETag": r["ETag"]})
        md5_bins.append(hashlib.md5(data).digest())

    complete = s3.complete_multipart_upload(
        Bucket=bucket, Key=key, UploadId=upload_id,
        MultipartUpload={"Parts": parts},
    )

    expected = hashlib.md5(b"".join(md5_bins)).hexdigest() + "-3"
    actual = complete["ETag"].strip('"')
    assert actual == expected


def test_part_etag_is_md5(s3, bucket):
    """Each part ETag should be MD5 of part content."""
    key = "part-etag.bin"
    resp = s3.create_multipart_upload(Bucket=bucket, Key=key)
    upload_id = resp["UploadId"]

    data = b"part etag test data"
    r = s3.upload_part(Bucket=bucket, Key=key, UploadId=upload_id, PartNumber=1, Body=data)

    expected = hashlib.md5(data).hexdigest()
    assert r["ETag"].strip('"') == expected

    s3.abort_multipart_upload(Bucket=bucket, Key=key, UploadId=upload_id)


def test_list_parts(s3, bucket):
    key = "list-parts.bin"
    resp = s3.create_multipart_upload(Bucket=bucket, Key=key)
    upload_id = resp["UploadId"]

    s3.upload_part(Bucket=bucket, Key=key, UploadId=upload_id, PartNumber=1, Body=b"aaa")
    s3.upload_part(Bucket=bucket, Key=key, UploadId=upload_id, PartNumber=2, Body=b"bbb")

    parts = s3.list_parts(Bucket=bucket, Key=key, UploadId=upload_id)
    assert len(parts["Parts"]) == 2

    s3.abort_multipart_upload(Bucket=bucket, Key=key, UploadId=upload_id)
