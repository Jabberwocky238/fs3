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
        rmp_serde::from_slice(&bytes).unwrap()
    }
}

impl From<XlMetaV2Version> for Vec<u8> {
    fn from(val: XlMetaV2Version) -> Self {
        rmp_serde::to_vec(&val).unwrap()
    }
}
