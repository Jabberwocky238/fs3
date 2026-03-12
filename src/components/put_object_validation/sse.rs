use axum::http::HeaderMap;

use crate::types::FS3Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseCustomerHeaders {
    pub algorithm: String,
    pub key_base64: String,
    pub key_md5_base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseKmsHeaders {
    pub key_id: Option<String>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedServerSideEncryption {
    S3 { algorithm: String },
    Customer(SseCustomerHeaders),
    Kms(SseKmsHeaders),
}

pub fn validate_sse_headers(
    headers: &HeaderMap,
) -> Result<Option<ParsedServerSideEncryption>, FS3Error> {
    let sse = header_value(headers, "x-amz-server-side-encryption")?;
    let sse_customer_algorithm =
        header_value(headers, "x-amz-server-side-encryption-customer-algorithm")?;
    let sse_customer_key =
        header_value(headers, "x-amz-server-side-encryption-customer-key")?;
    let sse_customer_key_md5 =
        header_value(headers, "x-amz-server-side-encryption-customer-key-md5")?;
    let kms_key_id = header_value(headers, "x-amz-server-side-encryption-aws-kms-key-id")?;
    let kms_context = header_value(headers, "x-amz-server-side-encryption-context")?;

    let customer_present = sse_customer_algorithm.is_some()
        || sse_customer_key.is_some()
        || sse_customer_key_md5.is_some();

    if customer_present && (sse.is_some() || kms_key_id.is_some() || kms_context.is_some()) {
        return Err(FS3Error::bad_request(
            "SSE-C headers cannot be combined with SSE-S3 or SSE-KMS headers",
        ));
    }

    if customer_present {
        let algorithm = sse_customer_algorithm
            .ok_or_else(|| FS3Error::bad_request("Missing SSE-C algorithm"))?;
        if !algorithm.eq_ignore_ascii_case("AES256") {
            return Err(FS3Error::bad_request("Unsupported SSE-C algorithm"));
        }
        let key_base64 =
            sse_customer_key.ok_or_else(|| FS3Error::bad_request("Missing SSE-C key"))?;
        let key_md5_base64 = sse_customer_key_md5
            .ok_or_else(|| FS3Error::bad_request("Missing SSE-C key MD5"))?;
        return Ok(Some(ParsedServerSideEncryption::Customer(
            SseCustomerHeaders {
                algorithm,
                key_base64,
                key_md5_base64,
            },
        )));
    }

    match sse.as_deref() {
        Some("AES256") => Ok(Some(ParsedServerSideEncryption::S3 {
            algorithm: "AES256".to_string(),
        })),
        Some("aws:kms") => Ok(Some(ParsedServerSideEncryption::Kms(SseKmsHeaders {
            key_id: kms_key_id,
            context: kms_context,
        }))),
        Some(_) => Err(FS3Error::bad_request(
            "Unsupported x-amz-server-side-encryption value",
        )),
        None if kms_key_id.is_some() || kms_context.is_some() => Err(FS3Error::bad_request(
            "KMS headers require x-amz-server-side-encryption=aws:kms",
        )),
        None => Ok(None),
    }
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
