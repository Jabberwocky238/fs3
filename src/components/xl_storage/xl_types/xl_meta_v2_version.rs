use rmp_serde::{Deserializer, Serializer};
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
        use rmpv::{Value, decode};
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
        rmp_serde::to_vec(&val).unwrap()
    }
}

impl From<&XlMetaV2Version> for Vec<u8> {
    fn from(val: &XlMetaV2Version) -> Self {
        val.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_1() {
        let expected = hex::decode("83a45479706501a556324f626ade0011a24944c4100102030405060708090a0b0c0d0e0f10a444446972c410aabbccddeeff11223344556677889900a64563416c676f00a345634d00a345634e00a745634253697a6500a74563496e64657800a645634469737490a84353756d416c676f00a8506172744e756d7390a9506172744554616773c0a95061727453697a657390aa506172744153697a6573c0a453697a65d10400a54d54696d65d2499602d2a74d657461537973c0a74d657461557372c0a17601").unwrap();
        let decoded: XlMetaV2Version = expected.clone().into();
        assert_eq!(decoded.version_type as u8, 1);
        assert_eq!(decoded.written_by_version, 1);
    }

    #[test]
    fn test_case_2() {
        let expected = hex::decode("83a45479706502a644656c4f626a82a24944c410ffeeddccbbaa99887766554433221100a54d54696d65d2499602d3a17602").unwrap();
        let decoded: XlMetaV2Version = expected.into();
        assert_eq!(decoded.version_type as u8, 2);
        assert_eq!(decoded.written_by_version, 2);
    }
}
