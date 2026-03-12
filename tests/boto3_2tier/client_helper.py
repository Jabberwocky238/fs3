#!/usr/bin/env python3
"""Shared boto3 client helper"""
import sys
import boto3
from botocore.config import Config

def create_client(endpoint):
    return boto3.client(
        "s3",
        endpoint_url=endpoint,
        aws_access_key_id="minioadmin",
        aws_secret_access_key="minioadmin",
        config=Config(signature_version="s3v4", s3={"addressing_style": "path"}),
        region_name="us-east-1",
    )

def get_endpoint():
    return sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:9000"
