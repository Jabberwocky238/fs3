"""Tests for bucket operations."""


def test_create_and_delete_bucket(s3):
    s3.create_bucket(Bucket="boto3-bucket-test")
    resp = s3.head_bucket(Bucket="boto3-bucket-test")
    assert resp["ResponseMetadata"]["HTTPStatusCode"] == 200
    s3.delete_bucket(Bucket="boto3-bucket-test")


def test_list_buckets(s3, bucket):
    resp = s3.list_buckets()
    names = [b["Name"] for b in resp["Buckets"]]
    assert bucket in names


def test_head_bucket(s3, bucket):
    resp = s3.head_bucket(Bucket=bucket)
    assert resp["ResponseMetadata"]["HTTPStatusCode"] == 200


def test_bucket_tagging(s3, bucket):
    tags = {"TagSet": [{"Key": "env", "Value": "test"}]}
    s3.put_bucket_tagging(Bucket=bucket, Tagging=tags)
    resp = s3.get_bucket_tagging(Bucket=bucket)
    assert resp["TagSet"][0]["Key"] == "env"
    s3.delete_bucket_tagging(Bucket=bucket)


def test_bucket_versioning(s3, bucket):
    s3.put_bucket_versioning(
        Bucket=bucket,
        VersioningConfiguration={"Status": "Enabled"},
    )
    resp = s3.get_bucket_versioning(Bucket=bucket)
    assert resp.get("Status") == "Enabled"


def test_bucket_policy(s3, bucket):
    import json
    policy = json.dumps({
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": "*",
            "Action": "s3:GetObject",
            "Resource": f"arn:aws:s3:::{bucket}/*",
        }],
    })
    s3.put_bucket_policy(Bucket=bucket, Policy=policy)
    resp = s3.get_bucket_policy(Bucket=bucket)
    assert "Statement" in resp["Policy"]
    s3.delete_bucket_policy(Bucket=bucket)
