#!/usr/bin/env python3
"""Test bucket replication operations"""
from client_helper import create_client, get_endpoint

def test(s3):
    bucket = "test-replication"
    s3.create_bucket(Bucket=bucket)

    replication = {
        "Role": "arn:aws:iam::123456789012:role/replication",
        "Rules": [{
            "ID": "rule1",
            "Status": "Enabled",
            "Priority": 1,
            "Filter": {},
            "Destination": {"Bucket": "arn:aws:s3:::dest-bucket"}
        }]
    }
    s3.put_bucket_replication(Bucket=bucket, ReplicationConfiguration=replication)
    print("[OK] PUT bucket replication")

    resp = s3.get_bucket_replication(Bucket=bucket)
    assert len(resp["Rules"]) == 1
    print("[OK] GET bucket replication")

    s3.delete_bucket_replication(Bucket=bucket)
    print("[OK] DELETE bucket replication")

    s3.delete_bucket(Bucket=bucket)

if __name__ == "__main__":
    endpoint = get_endpoint()
    print(f"Testing: {endpoint}")
    test(create_client(endpoint))
    print("[OK] All passed!")
