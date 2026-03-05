/// Object metadata - 对应 MinIO xlMetaV2Object

use super::types::{ErasureAlgo, ChecksumAlgo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[serde(rename = "PartETags")]
    pub part_etags: Vec<String>,
    #[serde(rename = "PartSizes")]
    pub part_sizes: Vec<i64>,
    #[serde(rename = "PartASizes", skip_serializing_if = "Vec::is_empty", default)]
    pub part_actual_sizes: Vec<i64>,
    #[serde(rename = "Size")]
    pub size: i64,
    #[serde(rename = "MTime")]
    pub mod_time: i64,
    #[serde(rename = "MetaSys", skip_serializing_if = "HashMap::is_empty", default)]
    pub meta_sys: HashMap<String, Vec<u8>>,
    #[serde(rename = "MetaUsr", skip_serializing_if = "HashMap::is_empty", default)]
    pub meta_user: HashMap<String, String>,
}
