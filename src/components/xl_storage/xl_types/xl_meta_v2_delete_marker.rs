use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV2DeleteMarker {
    #[serde(rename = "ID")]
    pub version_id: [u8; 16],
    #[serde(rename = "MTime")]
    pub mod_time: i64,
    #[serde(rename = "MetaSys", skip_serializing_if = "Option::is_none")]
    pub meta_sys: Option<HashMap<String, Vec<u8>>>,
}

impl From<Vec<u8>> for XlMetaV2DeleteMarker {
    fn from(bytes: Vec<u8>) -> Self {
        rmp_serde::from_slice(&bytes).unwrap()
    }
}

impl From<XlMetaV2DeleteMarker> for Vec<u8> {
    fn from(val: XlMetaV2DeleteMarker) -> Self {
        rmp_serde::to_vec(&val).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_1() {
        let obj = XlMetaV2DeleteMarker {
            version_id: [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16],
            mod_time: 1234567890,
            meta_sys: None,
        };
        let rust_bytes: Vec<u8> = obj.into();
        let expected = hex::decode("82a24944c4100102030405060708090a0b0c0d0e0f10a54d54696d65d2499602d2").unwrap();
        assert_eq!(rust_bytes, expected);
    }

    #[test]
    fn test_case_2() {
        let obj = XlMetaV2DeleteMarker {
            version_id: [0xff,0xee,0xdd,0xcc,0xbb,0xaa,0x99,0x88,0x77,0x66,0x55,0x44,0x33,0x22,0x11,0x00],
            mod_time: 1234567891,
            meta_sys: Some([("key1".to_string(),vec![0x01,0x02])].into()),
        };
        let rust_bytes: Vec<u8> = obj.into();
        let expected = hex::decode("83a24944c410ffeeddccbbaa99887766554433221100a54d54696d65d2499602d3a74d65746153797381a46b657931c4020102").unwrap();
        assert_eq!(rust_bytes, expected);
    }

    #[test]
    fn test_case_3() {
        let obj = XlMetaV2DeleteMarker {
            version_id: [0xde,0xad,0xbe,0xef,0xca,0xfe,0xba,0xbe,0x12,0x34,0x56,0x78,0x9a,0xbc,0xde,0xf0],
            mod_time: 0,
            meta_sys: None,
        };
        let rust_bytes: Vec<u8> = obj.into();
        let expected = hex::decode("82a24944c410deadbeefcafebabe123456789abcdef0a54d54696d6500").unwrap();
        assert_eq!(rust_bytes, expected);
    }
}
