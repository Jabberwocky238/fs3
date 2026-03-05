use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const XL_HEADER: [u8; 4] = *b"XL2 ";
pub const XL_VERSION_MAJOR: u16 = 1;
pub const XL_VERSION_MINOR: u16 = 3;

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
