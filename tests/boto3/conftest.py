"""
Shared fixtures for boto3 S3 compatibility tests.

Usage:
  1. Build the fs3 server: cargo build --release
  2. pip install boto3 pytest
  3. pytest tests/boto3/ -v
"""
import subprocess
import time
import socket
import os
import signal

import boto3
import pytest
from botocore.config import Config


def _free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


@pytest.fixture(scope="session")
def s3_endpoint():
    """Start fs3 server and return endpoint URL."""
    port = _free_port()
    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    bin_path = os.path.join(root, "target", "release", "fs3_test_server")
    if os.name == "nt":
        bin_path += ".exe"

    proc = subprocess.Popen(
        [bin_path, str(port)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    # Wait for server to be ready
    endpoint = f"http://127.0.0.1:{port}"
    for _ in range(30):
        try:
            socket.create_connection(("127.0.0.1", port), timeout=0.5).close()
            break
        except OSError:
            time.sleep(0.2)

    yield endpoint

    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()


@pytest.fixture(scope="session")
def s3(s3_endpoint):
    """Return a boto3 S3 client connected to the local fs3 server."""
    return boto3.client(
        "s3",
        endpoint_url=s3_endpoint,
        aws_access_key_id="minioadmin",
        aws_secret_access_key="minioadmin",
        config=Config(
            signature_version="s3v4",
            s3={"addressing_style": "path"},
            retries={"max_attempts": 0},
        ),
        region_name="us-east-1",
    )


@pytest.fixture()
def bucket(s3):
    """Create a temporary bucket and clean up after test."""
    name = f"test-{int(time.time() * 1000)}"
    s3.create_bucket(Bucket=name)
    yield name
    # cleanup
    try:
        objs = s3.list_objects_v2(Bucket=name).get("Contents", [])
        for obj in objs:
            s3.delete_object(Bucket=name, Key=obj["Key"])
        s3.delete_bucket(Bucket=name)
    except Exception:
        pass
