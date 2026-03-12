use super::msgpack_compat::{MsgpackReader, MsgpackWriter};
use super::xl_meta_v2_shallow_version::XlMetaV2ShallowVersion;
use super::xl_meta_v2_version_header::XlMetaV2VersionHeader;
use rmpv::{Value, decode::read_value};
use std::io::Cursor;

pub type XlMetaInlineData = Vec<u8>;

#[derive(Debug, Clone)]
pub struct XlMetaV2 {
    pub versions: Vec<XlMetaV2ShallowVersion>,
    pub inline_data: Option<XlMetaInlineData>,
    pub meta_v: u8,
}

impl From<Vec<u8>> for XlMetaV2 {
    fn from(bytes: Vec<u8>) -> Self {
        if bytes.len() > 13 && &bytes[0..4] == b"XL2 " && bytes[8] == 0xc6 {
            let payload_len = u32::from_be_bytes(bytes[9..13].try_into().unwrap()) as usize;
            let payload_start: usize = 13;
            let payload_end = payload_start.saturating_add(payload_len).min(bytes.len());
            let mut cursor = Cursor::new(&bytes[payload_start..payload_end]);

            let meta_v = read_value(&mut cursor)
                .ok()
                .and_then(|_| read_value(&mut cursor).ok())
                .and_then(|v| v.as_u64())
                .map(|v| v as u8)
                .unwrap_or(1);

            let versions_len = read_value(&mut cursor)
                .ok()
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;

            let mut versions = Vec::with_capacity(versions_len);
            for _ in 0..versions_len {
                let header = match read_value(&mut cursor).ok().and_then(value_into_bytes) {
                    Some(bytes) => bytes,
                    None => break,
                };
                let meta = match read_value(&mut cursor).ok().and_then(value_into_bytes) {
                    Some(bytes) => bytes,
                    None => break,
                };
                versions.push(XlMetaV2ShallowVersion {
                    header: header.into(),
                    meta,
                });
            }

            let inline_offset = payload_end + 5;
            let inline_data = if bytes.len() > inline_offset {
                Some(bytes[inline_offset..].to_vec())
            } else {
                None
            };

            Self {
                versions,
                inline_data,
                meta_v,
            }
        } else {
            let payload = if bytes.len() > 8 && &bytes[0..4] == b"XL2 " {
                &bytes[8..]
            } else {
                &bytes[..]
            };
            let r = MsgpackReader::new(payload);
            Self {
                versions: vec![],
                inline_data: r.get_bytes("Data"),
                meta_v: r.get_u8("MetaV").unwrap_or(1),
            }
        }
    }
}

fn value_into_bytes(value: Value) -> Option<Vec<u8>> {
    match value {
        Value::Binary(bytes) => Some(bytes),
        _ => value.as_slice().map(|bytes| bytes.to_vec()),
    }
}

impl From<XlMetaV2> for Vec<u8> {
    fn from(val: XlMetaV2) -> Self {
        let mut result = Vec::new();

        // XL2 header
        result.extend_from_slice(b"XL2 ");
        result.extend_from_slice(&[1u8, 0u8, 3u8, 0u8]);

        // Placeholder for bin32 size (will fill later)
        result.push(0xc6);
        result.extend_from_slice(&[0u8, 0u8, 0u8, 0u8]);
        let data_offset = result.len();

        // xlHeaderVersion (3)
        result.push(0x03);
        // xlMetaVersion (3)
        result.push(0x03);
        // versions count
        if val.versions.len() <= 127 {
            result.push(val.versions.len() as u8);
        } else {
            result.push(0xcd);
            result.extend_from_slice(&(val.versions.len() as u16).to_be_bytes());
        }

        // Serialize each version
        for v in &val.versions {
            // Serialize header using MinIO's tuple layout.
            let header_bytes: Vec<u8> = v.header.clone().into();

            // Append header as bin
            if header_bytes.len() <= 255 {
                result.push(0xc4);
                result.push(header_bytes.len() as u8);
            } else if header_bytes.len() <= 65535 {
                result.push(0xc5);
                result.extend_from_slice(&(header_bytes.len() as u16).to_be_bytes());
            } else {
                result.push(0xc6);
                result.extend_from_slice(&(header_bytes.len() as u32).to_be_bytes());
            }
            result.extend_from_slice(&header_bytes);

            // Append meta as bin
            if v.meta.len() <= 255 {
                result.push(0xc4);
                result.push(v.meta.len() as u8);
            } else if v.meta.len() <= 65535 {
                result.push(0xc5);
                result.extend_from_slice(&(v.meta.len() as u16).to_be_bytes());
            } else {
                result.push(0xc6);
                result.extend_from_slice(&(v.meta.len() as u32).to_be_bytes());
            }
            result.extend_from_slice(&v.meta);
        }

        // Update bin32 size
        let data_size = (result.len() - data_offset) as u32;
        result[data_offset - 4..data_offset].copy_from_slice(&data_size.to_be_bytes());

        // Add CRC (muint32)
        let crc = xxhash_rust::xxh64::xxh64(&result[data_offset..], 0) as u32;
        result.push(0xce);
        result.extend_from_slice(&crc.to_be_bytes());

        // Append inline data
        if let Some(ref data) = val.inline_data {
            result.extend_from_slice(data);
        }

        result
    }
}
