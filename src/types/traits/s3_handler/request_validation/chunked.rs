use axum::http::HeaderMap;

use crate::types::FS3Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamingSignatureAlgorithm {
    Aws4HmacSha256Payload,
    Aws4HmacSha256PayloadTrailer,
    Aws4EcdsaP256Sha256Payload,
    Aws4EcdsaP256Sha256PayloadTrailer,
}

impl StreamingSignatureAlgorithm {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "STREAMING-AWS4-HMAC-SHA256-PAYLOAD" => Some(Self::Aws4HmacSha256Payload),
            "STREAMING-AWS4-HMAC-SHA256-PAYLOAD-TRAILER" => {
                Some(Self::Aws4HmacSha256PayloadTrailer)
            }
            "STREAMING-AWS4-ECDSA-P256-SHA256-PAYLOAD" => {
                Some(Self::Aws4EcdsaP256Sha256Payload)
            }
            "STREAMING-AWS4-ECDSA-P256-SHA256-PAYLOAD-TRAILER" => {
                Some(Self::Aws4EcdsaP256Sha256PayloadTrailer)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwsChunkedUpload {
    pub algorithm: StreamingSignatureAlgorithm,
    pub decoded_content_length: u64,
    pub trailer: Option<String>,
}

pub fn parse_aws_chunked_upload(
    headers: &HeaderMap,
) -> Result<Option<AwsChunkedUpload>, FS3Error> {
    let content_encoding = header_value(headers, "content-encoding")?;
    let Some(content_encoding) = content_encoding else {
        return Ok(None);
    };

    let has_aws_chunked = content_encoding
        .split(',')
        .any(|part| part.trim().eq_ignore_ascii_case("aws-chunked"));
    if !has_aws_chunked {
        return Ok(None);
    }

    let payload_hash = header_value(headers, "x-amz-content-sha256")?
        .ok_or_else(|| FS3Error::bad_request("Missing x-amz-content-sha256 for aws-chunked"))?;
    let algorithm = StreamingSignatureAlgorithm::parse(payload_hash.as_str())
        .ok_or_else(|| FS3Error::bad_request("Unsupported x-amz-content-sha256 streaming mode"))?;

    let decoded_content_length = decode_decoded_content_length(headers)?;
    let trailer = header_value(headers, "x-amz-trailer")?;

    Ok(Some(AwsChunkedUpload {
        algorithm,
        decoded_content_length,
        trailer,
    }))
}

pub fn decode_decoded_content_length(headers: &HeaderMap) -> Result<u64, FS3Error> {
    let value = header_value(headers, "x-amz-decoded-content-length")?
        .ok_or_else(|| FS3Error::bad_request("Missing x-amz-decoded-content-length"))?;
    value
        .parse::<u64>()
        .map_err(|_| FS3Error::bad_request("Invalid x-amz-decoded-content-length"))
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
