"""Tests for object tagging operations."""


def test_put_and_get_object_tagging(s3, bucket):
    s3.put_object(Bucket=bucket, Key="tagged.txt", Body=b"tagged")
    s3.put_object_tagging(
        Bucket=bucket, Key="tagged.txt",
        Tagging={"TagSet": [{"Key": "env", "Value": "prod"}]},
    )
    resp = s3.get_object_tagging(Bucket=bucket, Key="tagged.txt")
    assert len(resp["TagSet"]) == 1
    assert resp["TagSet"][0]["Key"] == "env"
    assert resp["TagSet"][0]["Value"] == "prod"


def test_delete_object_tagging(s3, bucket):
    s3.put_object(Bucket=bucket, Key="tagged2.txt", Body=b"tagged")
    s3.put_object_tagging(
        Bucket=bucket, Key="tagged2.txt",
        Tagging={"TagSet": [{"Key": "k", "Value": "v"}]},
    )
    s3.delete_object_tagging(Bucket=bucket, Key="tagged2.txt")
    resp = s3.get_object_tagging(Bucket=bucket, Key="tagged2.txt")
    assert len(resp["TagSet"]) == 0
