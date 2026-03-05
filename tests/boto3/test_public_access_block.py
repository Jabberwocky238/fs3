#!/usr/bin/env python3
"""Test public access block API"""
from client_helper import create_client, get_endpoint

def test_public_access_block():
    s3 = create_client(get_endpoint())
    bucket = "test-public-access"

    s3.create_bucket(Bucket=bucket)

    # Put public access block
    s3.put_public_access_block(
        Bucket=bucket,
        PublicAccessBlockConfiguration={
            'BlockPublicAcls': True,
            'IgnorePublicAcls': True,
            'BlockPublicPolicy': True,
            'RestrictPublicBuckets': True
        }
    )
    print("✓ put_public_access_block")

    # Get public access block
    resp = s3.get_public_access_block(Bucket=bucket)
    config = resp['PublicAccessBlockConfiguration']
    print(f"✓ get_public_access_block: BlockPublicAcls={config['BlockPublicAcls']}")

    # Delete public access block
    s3.delete_public_access_block(Bucket=bucket)
    print("✓ delete_public_access_block")

    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_public_access_block()
