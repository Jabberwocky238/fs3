use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaDataDirDecoder {
    #[serde(rename = "V2Obj", skip_serializing_if = "Option::is_none")]
    pub object_v2: Option<XlMetaDataDirDecoderObjectV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaDataDirDecoderObjectV2 {
    #[serde(rename = "DDir")]
    pub data_dir: [u8; 16],
}

impl From<Vec<u8>> for XlMetaDataDirDecoder {
    fn from(bytes: Vec<u8>) -> Self {
        rmp_serde::from_slice(&bytes).unwrap()
    }
}

impl From<XlMetaDataDirDecoder> for Vec<u8> {
    fn from(val: XlMetaDataDirDecoder) -> Self {
        rmp_serde::to_vec(&val).unwrap()
    }
}
