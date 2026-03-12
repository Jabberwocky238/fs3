use serde::{Deserialize, Serialize};

use super::xl_meta_v2_delete_marker::XlMetaV2DeleteMarker;
use super::xl_meta_v2_object::XlMetaV2Object;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum VersionType {
    #[serde(rename = "0")]
    Invalid = 0,
    #[serde(rename = "1")]
    Object = 1,
    #[serde(rename = "2")]
    Delete = 2,
    #[serde(rename = "3")]
    Legacy = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV2Version {
    #[serde(rename = "Type")]
    pub version_type: VersionType,
    #[serde(rename = "V1Obj", skip_serializing_if = "Option::is_none")]
    pub object_v1: Option<Vec<u8>>,
    #[serde(rename = "V2Obj", skip_serializing_if = "Option::is_none")]
    pub object_v2: Option<XlMetaV2Object>,
    #[serde(rename = "DelObj", skip_serializing_if = "Option::is_none")]
    pub delete_marker: Option<XlMetaV2DeleteMarker>,
    #[serde(rename = "v")]
    pub written_by_version: u64,
}

impl From<Vec<u8>> for XlMetaV2Version {
    fn from(bytes: Vec<u8>) -> Self {
        use rmpv::decode;
        let val = decode::read_value(&mut &bytes[..]).unwrap();
        let map = val.as_map().unwrap();

        let mut version_type = VersionType::Invalid;
        let mut object_v1 = None;
        let mut object_v2 = None;
        let mut delete_marker = None;
        let mut written_by_version = 0;

        for (k, v) in map {
            match k.as_str().unwrap() {
                "Type" => version_type = unsafe { std::mem::transmute(v.as_u64().unwrap() as u8) },
                "V1Obj" => {
                    let mut buf = Vec::new();
                    rmpv::encode::write_value(&mut buf, v).unwrap();
                    object_v1 = Some(buf);
                }
                "V2Obj" => {
                    let mut buf = Vec::new();
                    rmpv::encode::write_value(&mut buf, v).unwrap();
                    object_v2 = Some(buf.into());
                }
                "DelObj" => {
                    let mut buf = Vec::new();
                    rmpv::encode::write_value(&mut buf, v).unwrap();
                    delete_marker = Some(buf.into());
                }
                "v" => written_by_version = v.as_u64().unwrap(),
                _ => {}
            }
        }

        Self { version_type, object_v1, object_v2, delete_marker, written_by_version }
    }
}

impl From<XlMetaV2Version> for Vec<u8> {
    fn from(val: XlMetaV2Version) -> Self {
        let mut buf = Vec::new();
        let mut field_count = 2;
        if val.object_v1.is_some() {
            field_count += 1;
        }
        if val.object_v2.is_some() {
            field_count += 1;
        }
        if val.delete_marker.is_some() {
            field_count += 1;
        }

        rmp::encode::write_map_len(&mut buf, field_count).unwrap();
        rmp::encode::write_str(&mut buf, "Type").unwrap();
        rmp::encode::write_uint(&mut buf, val.version_type as u64).unwrap();

        if let Some(object_v1) = val.object_v1 {
            rmp::encode::write_str(&mut buf, "V1Obj").unwrap();
            buf.extend_from_slice(&object_v1);
        }
        if let Some(object_v2) = val.object_v2 {
            rmp::encode::write_str(&mut buf, "V2Obj").unwrap();
            let object_bytes: Vec<u8> = object_v2.into();
            buf.extend_from_slice(&object_bytes);
        }
        if let Some(delete_marker) = val.delete_marker {
            rmp::encode::write_str(&mut buf, "DelObj").unwrap();
            let delete_marker_bytes: Vec<u8> = delete_marker.into();
            buf.extend_from_slice(&delete_marker_bytes);
        }

        rmp::encode::write_str(&mut buf, "v").unwrap();
        rmp::encode::write_uint(&mut buf, val.written_by_version).unwrap();
        buf
    }
}

impl From<&XlMetaV2Version> for Vec<u8> {
    fn from(val: &XlMetaV2Version) -> Self {
        val.clone().into()
    }
}
