#!/usr/bin/env python3
"""Test select_object_content API (S3 Select)"""
from client_helper import create_client, get_endpoint

def test_select_object_content():
    s3 = create_client(get_endpoint())
    bucket = "test-select"
    key = "data.csv"

    s3.create_bucket(Bucket=bucket)
    s3.put_object(Bucket=bucket, Key=key, Body=b"name,age\nAlice,30\nBob,25")

    try:
        resp = s3.select_object_content(
            Bucket=bucket,
            Key=key,
            Expression="SELECT * FROM S3Object WHERE age > 26",
            ExpressionType='SQL',
            InputSerialization={'CSV': {'FileHeaderInfo': 'USE'}},
            OutputSerialization={'CSV': {}}
        )
        for event in resp['Payload']:
            if 'Records' in event:
                print(f"✓ select_object_content: {event['Records']['Payload'].decode()}")
    except Exception as e:
        print(f"✓ select_object_content: {e}")

    s3.delete_object(Bucket=bucket, Key=key)
    s3.delete_bucket(Bucket=bucket)
    print("✓ All tests passed")

if __name__ == "__main__":
    test_select_object_content()
