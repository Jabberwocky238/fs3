"""Tests for listing objects."""


def test_list_objects_v2(s3, bucket):
    s3.put_object(Bucket=bucket, Key="a.txt", Body=b"a")
    s3.put_object(Bucket=bucket, Key="b.txt", Body=b"b")
    resp = s3.list_objects_v2(Bucket=bucket)
    keys = [o["Key"] for o in resp.get("Contents", [])]
    assert "a.txt" in keys
    assert "b.txt" in keys


def test_list_objects_v2_prefix(s3, bucket):
    s3.put_object(Bucket=bucket, Key="dir/x.txt", Body=b"x")
    s3.put_object(Bucket=bucket, Key="dir/y.txt", Body=b"y")
    s3.put_object(Bucket=bucket, Key="other.txt", Body=b"o")
    resp = s3.list_objects_v2(Bucket=bucket, Prefix="dir/")
    keys = [o["Key"] for o in resp.get("Contents", [])]
    assert len(keys) == 2
    assert all(k.startswith("dir/") for k in keys)


def test_delete_objects(s3, bucket):
    s3.put_object(Bucket=bucket, Key="d1.txt", Body=b"1")
    s3.put_object(Bucket=bucket, Key="d2.txt", Body=b"2")
    resp = s3.delete_objects(
        Bucket=bucket,
        Delete={"Objects": [{"Key": "d1.txt"}, {"Key": "d2.txt"}]},
    )
    assert len(resp.get("Deleted", [])) == 2
