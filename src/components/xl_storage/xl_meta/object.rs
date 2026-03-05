/// Object metadata - 对应 MinIO xlMetaV2Object

use super::types::{ErasureAlgo, ChecksumAlgo};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::collections::HashMap;

fn serialize_bytes_as_bin<S>(bytes: &[u8; 16], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_bytes(bytes)
}

fn deserialize_bytes_from_bin<'de, D>(d: D) -> Result<[u8; 16], D::Error>
where
    D: Deserializer<'de>,
{
    let bytes = <Vec<u8>>::deserialize(d)?;
    bytes.try_into().map_err(|_| serde::de::Error::custom("expected 16 bytes"))
}

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
    #[serde(rename = "ID", serialize_with = "serialize_bytes_as_bin", deserialize_with = "deserialize_bytes_from_bin")]
    pub version_id: [u8; 16],
    #[serde(rename = "DDir", serialize_with = "serialize_bytes_as_bin", deserialize_with = "deserialize_bytes_from_bin")]
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

impl From<&XlMetaV2Object> for super::golang_struct::GoBytes {
    fn from(obj: &XlMetaV2Object) -> Self {
        use super::golang_struct::GoStructBuilder;
        let mut b = GoStructBuilder::new(17);

        b.field_bin("ID", &obj.version_id);
        b.field_bin("DDir", &obj.data_dir);
        b.field_u8("EcAlgo", obj.erasure_algorithm as u8);
        b.field_i32("EcM", obj.erasure_m);
        b.field_i32("EcN", obj.erasure_n);
        b.field_i32("EcBSize", obj.erasure_block_size as i32);
        b.field_i32("EcIndex", obj.erasure_index);
        b.field_array_u8("EcDist", &obj.erasure_dist);
        b.field_u8("CSumAlgo", obj.checksum_algo as u8);
        b.field_array_i32("PartNums", &obj.part_numbers);

        if obj.part_etags.is_empty() {
            b.field_nil("PartETags");
        } else {
            b.field_array_str("PartETags", &obj.part_etags);
        }

        b.field_array_i64("PartSizes", &obj.part_sizes);
        b.field_array_i64("PartASizes", &obj.part_actual_sizes);
        b.field_i64("Size", obj.size);
        b.field_i64_as_i32("MTime", obj.mod_time);
        b.field_map_str_bin("MetaSys", &obj.meta_sys);
        b.field_map_str_str("MetaUsr", &obj.meta_user);

        b.build()
    }
}
