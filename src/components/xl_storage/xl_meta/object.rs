/// Object metadata - 对应 MinIO xlMetaV2Object

use super::types::{ErasureAlgo, ChecksumAlgo};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::collections::HashMap;

fn serialize_vec_as_nil<S>(v: &Vec<String>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if v.is_empty() {
        s.serialize_none()
    } else {
        v.serialize(s)
    }
}

fn deserialize_vec_from_nil<'de, D>(d: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<Vec<String>>::deserialize(d).map(|opt| opt.unwrap_or_default())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV2Object {
    #[serde(rename = "ID")]
    pub version_id: [u8; 16],
    #[serde(rename = "DDir")]
    pub data_dir: [u8; 16],
    #[serde(rename = "EcAlgo")]
    pub erasure_algorithm: ErasureAlgo,
    #[serde(rename = "EcM")]
    pub erasure_m: i32,
    #[serde(rename = "EcN")]
    pub erasure_n: i32,
    #[serde(rename = "EcBSize")]
    pub erasure_block_size: i64,
    #[serde(rename = "EcIndex")]
    pub erasure_index: i32,
    #[serde(rename = "EcDist")]
    pub erasure_dist: Vec<u8>,
    #[serde(rename = "CSumAlgo")]
    pub checksum_algo: ChecksumAlgo,
    #[serde(rename = "PartNums")]
    pub part_numbers: Vec<i32>,
    #[serde(rename = "PartETags", serialize_with = "serialize_vec_as_nil", deserialize_with = "deserialize_vec_from_nil")]
    pub part_etags: Vec<String>,
    #[serde(rename = "PartSizes")]
    pub part_sizes: Vec<i64>,
    #[serde(rename = "PartASizes")]
    pub part_actual_sizes: Vec<i64>,
    #[serde(rename = "PartIdx", skip_serializing_if = "Vec::is_empty", default)]
    pub part_indices: Vec<Vec<u8>>,
    #[serde(rename = "Size")]
    pub size: i64,
    #[serde(rename = "MTime")]
    pub mod_time: i64,
    #[serde(rename = "MetaSys")]
    pub meta_sys: HashMap<String, Vec<u8>>,
    #[serde(rename = "MetaUsr")]
    pub meta_user: HashMap<String, String>,
}

impl XlMetaV2Object {
    pub fn signature(&self) -> [u8; 4] {
        use crate::components::xl_storage::xl_meta::hash_deterministic_string;
        let crc = hash_deterministic_string(&self.meta_user);
        let mut sig = [0u8; 4];
        sig.copy_from_slice(&(crc as u32).to_le_bytes());
        sig
    }
}
