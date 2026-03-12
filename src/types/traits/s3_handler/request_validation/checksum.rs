use axum::http::HeaderMap;
use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::types::FS3Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    Sha256,
    Sha1,
    Crc32,
    Crc32c,
}

impl ChecksumAlgorithm {
    pub fn header_name(self) -> &'static str {
        match self {
            Self::Sha256 => "x-amz-checksum-sha256",
            Self::Sha1 => "x-amz-checksum-sha1",
            Self::Crc32 => "x-amz-checksum-crc32",
            Self::Crc32c => "x-amz-checksum-crc32c",
        }
    }

    pub fn digest_len(self) -> usize {
        match self {
            Self::Sha256 => 32,
            Self::Sha1 => 20,
            Self::Crc32 | Self::Crc32c => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentMd5Expectation {
    pub base64_value: String,
    pub raw: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecksumExpectation {
    pub algorithm: ChecksumAlgorithm,
    pub header_name: &'static str,
    pub base64_value: String,
    pub raw: Vec<u8>,
}

pub fn decode_content_md5(value: &str) -> Result<ContentMd5Expectation, FS3Error> {
    let raw = STANDARD
        .decode(value.trim())
        .map_err(|_| FS3Error::bad_request("InvalidDigest"))?;
    let raw: [u8; 16] = raw
        .try_into()
        .map_err(|_| FS3Error::bad_request("InvalidDigest"))?;
    Ok(ContentMd5Expectation {
        base64_value: value.trim().to_string(),
        raw,
    })
}

pub fn parse_checksum_headers(
    headers: &HeaderMap,
) -> Result<Option<ChecksumExpectation>, FS3Error> {
    let mut found = Vec::new();
    for algorithm in [
        ChecksumAlgorithm::Sha256,
        ChecksumAlgorithm::Sha1,
        ChecksumAlgorithm::Crc32,
        ChecksumAlgorithm::Crc32c,
    ] {
        if let Some(value) = header_value(headers, algorithm.header_name())? {
            let raw = STANDARD
                .decode(value.as_bytes())
                .map_err(|_| FS3Error::bad_request("InvalidRequest"))?;
            if raw.len() != algorithm.digest_len() {
                return Err(FS3Error::bad_request("InvalidRequest"));
            }
            found.push(ChecksumExpectation {
                algorithm,
                header_name: algorithm.header_name(),
                base64_value: value,
                raw,
            });
        }
    }

    if found.len() > 1 {
        return Err(FS3Error::bad_request(
            "Only one x-amz-checksum-* header may be specified",
        ));
    }

    Ok(found.into_iter().next())
}

fn header_value(headers: &HeaderMap, name: &str) -> Result<Option<String>, FS3Error> {
    headers
        .get(name)
        .map(|v| {
            v.to_str()
                .map(|s| s.trim().to_string())
                .map_err(|_| FS3Error::bad_request("InvalidRequest"))
        })
        .transpose()
}
