use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const XL_HEADER: [u8; 4] = *b"XL2 ";
const XL_VERSION_MAJOR: u16 = 1;
const XL_VERSION_MINOR: u16 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV2Object {
    #[serde(rename = "ID")]
    pub version_id: [u8; 16],
    #[serde(rename = "DDir")]
    pub data_dir: [u8; 16],
    #[serde(rename = "EcAlgo")]
    pub ec_algo: u8,
    #[serde(rename = "EcM")]
    pub ec_m: i32,
    #[serde(rename = "EcN")]
    pub ec_n: i32,
    #[serde(rename = "EcBSize")]
    pub ec_bsize: i64,
    #[serde(rename = "EcIndex")]
    pub ec_index: i32,
    #[serde(rename = "EcDist")]
    pub ec_dist: Vec<u8>,
    #[serde(rename = "CSumAlgo")]
    pub csum_algo: u8,
    #[serde(rename = "PartNums")]
    pub part_nums: Vec<i32>,
    #[serde(rename = "PartETags")]
    pub part_etags: Option<Vec<String>>,
    #[serde(rename = "PartSizes")]
    pub part_sizes: Vec<i64>,
    #[serde(rename = "PartASizes")]
    pub part_asizes: Vec<i64>,
    #[serde(rename = "Size")]
    pub size: i64,
    #[serde(rename = "MTime")]
    pub mod_time: i64,
    #[serde(rename = "MetaSys")]
    pub meta_sys: HashMap<String, Vec<u8>>,
    #[serde(rename = "MetaUsr")]
    pub meta_user: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV2Version {
    #[serde(rename = "Type")]
    pub version_type: u8,
    #[serde(rename = "V2Obj")]
    pub object_v2: Option<XlMetaV2Object>,
}

pub struct XlMetaV2 {
    pub versions: Vec<XlMetaV2Version>,
    pub inline_data: Vec<u8>,
}

impl XlMetaV2 {
    pub fn encode(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use rmp::encode;
        let mut buf = Vec::new();

        buf.extend_from_slice(&XL_HEADER);
        buf.extend_from_slice(&XL_VERSION_MAJOR.to_le_bytes());
        buf.extend_from_slice(&XL_VERSION_MINOR.to_le_bytes());
        buf.push(0xc6);
        let size_offset = buf.len();
        buf.extend_from_slice(&[0u8; 4]);
        let data_offset = buf.len();

        encode::write_uint(&mut buf, 3)?;
        encode::write_uint(&mut buf, 3)?;
        encode::write_array_len(&mut buf, self.versions.len() as u32)?;

        for ver in &self.versions {
            let full_bytes = rmp_serde::to_vec(&ver)?;
            encode::write_bin(&mut buf, &full_bytes)?;
            encode::write_bin(&mut buf, &full_bytes)?;
        }

        let payload_size = (buf.len() - data_offset) as u32;
        buf[size_offset..size_offset + 4].copy_from_slice(&payload_size.to_be_bytes());

        let crc = crc32fast::hash(&buf[data_offset..]);
        buf.push(0xce);
        buf.extend_from_slice(&crc.to_be_bytes());
        buf.extend_from_slice(&self.inline_data);

        Ok(buf)
    }

    pub fn decode(buf: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if buf.len() < 8 || &buf[0..4] != &XL_HEADER {
            return Err("invalid xl.meta header".into());
        }

        let mut pos = 8;
        if buf[pos] != 0xc6 {
            return Err("invalid msgpack format".into());
        }
        pos += 1;
        let payload_size = u32::from_be_bytes([buf[pos], buf[pos+1], buf[pos+2], buf[pos+3]]) as usize;
        pos += 4;
        let payload_start = pos;

        let inline_start = pos + payload_size + 5;
        let inline_data = if inline_start < buf.len() {
            buf[inline_start..].to_vec()
        } else {
            Vec::new()
        };

        use rmp::decode;
        let mut cursor = &buf[payload_start..];
        let _header_ver: u64 = decode::read_int(&mut cursor)?;
        let _meta_ver: u64 = decode::read_int(&mut cursor)?;
        let ver_count = decode::read_array_len(&mut cursor)? as usize;

        let mut versions = Vec::new();
        for _ in 0..ver_count {
            let header_len = decode::read_bin_len(&mut cursor)? as usize;
            cursor = &cursor[header_len..];
            let ver_len = decode::read_bin_len(&mut cursor)? as usize;
            if ver_len > 0 && cursor.len() >= ver_len {
                match rmp_serde::from_slice::<XlMetaV2Version>(&cursor[..ver_len]) {
                    Ok(ver) => versions.push(ver),
                    Err(_) => {}
                }
                cursor = &cursor[ver_len..];
            }
        }

        Ok(XlMetaV2 { versions, inline_data })
    }
}
