#!/usr/bin/env python3
"""Test bucket policy status API"""
from client_helper import create_client, get_endpoint
import json

def test_policy_status():
    s3 = create_client(get_endpoint())
    bucket = "test-policy-status"

    s3.create_bucket(Bucket=bucket)

    # Put a public policy
    policy = {
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": "*",
            "Action": "s3:GetObject",
            "Resource": f"arn:aws:s3:::{bucket}/*"
        }]
    }
    s3.put_bucket_policy(Bucket=bucket, Policy=json.dumps(policy))
    print("✓ put_bucket_policy")

    # Get policy status
    resp = s3.get_bucket_policy_status(Bucket=bucket)
    print(f"✓ get_bucket_policy_status: IsPublic={resp['PolicyStatus']['IsPublic']}")

    s3.delete_bucket_policy(Bucket=bucket)
    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_policy_status()
