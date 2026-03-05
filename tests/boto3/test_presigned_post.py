#!/usr/bin/env python3
"""Test generate_presigned_post API"""
from client_helper import create_client, get_endpoint

def test_presigned_post():
    s3 = create_client(get_endpoint())
    bucket = "test-presigned-post"
    key = "uploaded.txt"

    s3.create_bucket(Bucket=bucket)

    # Generate presigned POST
    resp = s3.generate_presigned_post(
        Bucket=bucket,
        Key=key,
        ExpiresIn=3600
    )
    print(f"✓ generate_presigned_post: {resp['url']}")
    print(f"  Fields: {list(resp['fields'].keys())}")

    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_presigned_post()
