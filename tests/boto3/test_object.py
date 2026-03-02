"""Tests for single object operations."""
import hashlib


def test_put_and_get_object(s3, bucket):
    body = b"Hello FS3"
    s3.put_object(Bucket=bucket, Key="hello.txt", Body=body)
    resp = s3.get_object(Bucket=bucket, Key="hello.txt")
    assert resp["Body"].read() == body


def test_put_object_etag(s3, bucket):
    body = b"etag test content"
    resp = s3.put_object(Bucket=bucket, Key="etag.txt", Body=body)
    expected = hashlib.md5(body).hexdigest()
    # ETag comes back with quotes
    assert resp["ETag"] == f'"{expected}"'


def test_head_object(s3, bucket):
    s3.put_object(Bucket=bucket, Key="head.txt", Body=b"head")
    resp = s3.head_object(Bucket=bucket, Key="head.txt")
    assert resp["ContentLength"] == 4
    assert "ETag" in resp


def test_delete_object(s3, bucket):
    s3.put_object(Bucket=bucket, Key="del.txt", Body=b"delete me")
    s3.delete_object(Bucket=bucket, Key="del.txt")
    import botocore.exceptions
    try:
        s3.head_object(Bucket=bucket, Key="del.txt")
        assert False, "Object should be deleted"
    except botocore.exceptions.ClientError as e:
        assert e.response["ResponseMetadata"]["HTTPStatusCode"] in (404, 403)


def test_copy_object(s3, bucket):
    body = b"copy source"
    s3.put_object(Bucket=bucket, Key="src.txt", Body=body)
    s3.copy_object(
        Bucket=bucket,
        Key="dst.txt",
        CopySource={"Bucket": bucket, "Key": "src.txt"},
    )
    resp = s3.get_object(Bucket=bucket, Key="dst.txt")
    assert resp["Body"].read() == body


def test_copy_object_preserves_etag(s3, bucket):
    body = b"etag copy test"
    put_resp = s3.put_object(Bucket=bucket, Key="src2.txt", Body=body)
    copy_resp = s3.copy_object(
        Bucket=bucket,
        Key="dst2.txt",
        CopySource={"Bucket": bucket, "Key": "src2.txt"},
    )
    head_resp = s3.head_object(Bucket=bucket, Key="dst2.txt")
    assert head_resp["ETag"] == put_resp["ETag"]
