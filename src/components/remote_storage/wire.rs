use std::collections::HashMap;
use std::io::Cursor;

use rmp::encode::{write_bin, write_bool, write_map_len, write_str};
use rmpv::Value;
use serde::{Deserialize, Serialize};

use crate::types::s3::storage_types::{DeletePathOptions, FileInfo, RenameDataResult};

pub const STORAGE_REST_VERSION: &str = "v63";
pub const STORAGE_REST_PREFIX: &str = "/minio/storage";

pub const STORAGE_REST_METHOD_HEALTH: &str = "/health";
pub const STORAGE_REST_METHOD_CREATE_FILE: &str = "/cfile";
pub const STORAGE_REST_METHOD_READ_VERSION: &str = "/rver";

pub const STORAGE_REST_PARAM_VOLUME: &str = "vol";
pub const STORAGE_REST_PARAM_FILE_PATH: &str = "fp";
pub const STORAGE_REST_PARAM_VERSION_ID: &str = "vid";
pub const STORAGE_REST_PARAM_DISK_ID: &str = "did";
pub const STORAGE_REST_PARAM_SRC_VOLUME: &str = "svol";
pub const STORAGE_REST_PARAM_SRC_PATH: &str = "spath";
pub const STORAGE_REST_PARAM_DST_VOLUME: &str = "dvol";
pub const STORAGE_REST_PARAM_DST_PATH: &str = "dpath";
pub const STORAGE_REST_PARAM_ORIG_VOLUME: &str = "ovol";
pub const STORAGE_REST_PARAM_LENGTH: &str = "length";
pub const STORAGE_REST_PARAM_HEALING: &str = "heal";
pub const STORAGE_REST_PARAM_INCLUDE_FREE_VERSIONS: &str = "incl-fv";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinioStorageCall {
    ReadVersion,
    WriteAll,
    RenameData,
    Delete,
}

impl MinioStorageCall {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadVersion => "ReadVersion",
            Self::WriteAll => "WriteAll",
            Self::RenameData => "RenameData",
            Self::Delete => "Delete",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinioReadVersionRequest {
    #[serde(rename = "id")]
    pub disk_id: String,
    #[serde(rename = "ovol")]
    pub orig_volume: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "fp")]
    pub file_path: String,
    #[serde(rename = "vid")]
    pub version_id: String,
    #[serde(rename = "heal")]
    pub healing: bool,
    #[serde(rename = "incl-fv")]
    pub incl_free_versions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinioWriteAllRequest {
    #[serde(rename = "id")]
    pub disk_id: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "fp")]
    pub file_path: String,
    #[serde(rename = "b")]
    pub buf: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinioDeleteOptions {
    #[serde(rename = "r")]
    pub recursive: bool,
    #[serde(rename = "i")]
    pub immediate: bool,
    #[serde(rename = "u")]
    pub undo_write: bool,
    #[serde(rename = "o")]
    pub old_data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinioDeleteFileRequest {
    #[serde(rename = "id")]
    pub disk_id: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "fp")]
    pub file_path: String,
    #[serde(rename = "do")]
    pub opts: MinioDeleteOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinioRenameOptions {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinioRenameDataRequest {
    #[serde(rename = "id")]
    pub disk_id: String,
    #[serde(rename = "sv")]
    pub src_volume: String,
    #[serde(rename = "sp")]
    pub src_path: String,
    #[serde(rename = "dv")]
    pub dst_volume: String,
    #[serde(rename = "dp")]
    pub dst_path: String,
    #[serde(rename = "fi")]
    pub file_info: FileInfo,
    #[serde(rename = "ro")]
    pub opts: MinioRenameOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinioRenameDataResponse {
    #[serde(rename = "s")]
    pub sign: Vec<u8>,
    #[serde(rename = "od")]
    pub old_data_dir: String,
}

pub fn encode_minio_write_all_request(req: &MinioWriteAllRequest) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    write_map_len(&mut out, 4).map_err(|err| err.to_string())?;
    write_string_entry(&mut out, "id", &req.disk_id)?;
    write_string_entry(&mut out, "v", &req.volume)?;
    write_string_entry(&mut out, "fp", &req.file_path)?;
    write_bytes_entry(&mut out, "b", &req.buf)?;
    Ok(out)
}

pub fn encode_minio_delete_file_request(req: &MinioDeleteFileRequest) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    write_map_len(&mut out, 4).map_err(|err| err.to_string())?;
    write_string_entry(&mut out, "id", &req.disk_id)?;
    write_string_entry(&mut out, "v", &req.volume)?;
    write_string_entry(&mut out, "fp", &req.file_path)?;

    write_str(&mut out, "do").map_err(|err| err.to_string())?;
    write_map_len(&mut out, 4).map_err(|err| err.to_string())?;
    write_bool_entry(&mut out, "r", req.opts.recursive)?;
    write_bool_entry(&mut out, "i", req.opts.immediate)?;
    write_bool_entry(&mut out, "u", req.opts.undo_write)?;
    write_string_entry(&mut out, "o", &req.opts.old_data_dir)?;
    Ok(out)
}

impl From<DeletePathOptions> for MinioDeleteOptions {
    fn from(value: DeletePathOptions) -> Self {
        Self {
            recursive: value.recursive,
            immediate: false,
            undo_write: false,
            old_data_dir: String::new(),
        }
    }
}

impl MinioRenameDataResponse {
    pub fn into_fs3_result(self, dst_path: &str) -> RenameDataResult {
        let old_data_dir = if self.old_data_dir.is_empty() {
            None
        } else {
            Some(self.old_data_dir)
        };
        let old_data_path = old_data_dir
            .as_ref()
            .map(|old_data_dir| format!("{dst_path}/{old_data_dir}"));

        RenameDataResult {
            old_data_dir,
            old_data_path,
            cleanup_src_volume: String::new(),
            cleanup_src_path: String::new(),
        }
    }
}

pub fn decode_minio_file_info(bytes: &[u8]) -> Result<FileInfo, String> {
    let value = rmpv::decode::read_value(&mut Cursor::new(bytes))
        .map_err(|err| format!("failed to decode MinIO msgpack FileInfo: {err}"))?;
    let tuple = value
        .as_array()
        .ok_or_else(|| "MinIO FileInfo is not encoded as a tuple".to_string())?;
    if tuple.len() < 19 {
        return Err(format!(
            "MinIO FileInfo tuple is too short: expected at least 19 fields, got {}",
            tuple.len()
        ));
    }

    let metadata = value_string_map(&tuple[16]);
    let erasure = decode_minio_erasure_info(&tuple[18])?;

    Ok(FileInfo {
        volume: tuple_string(tuple, 0),
        name: tuple_string(tuple, 1),
        version_id: tuple_string(tuple, 2),
        size: tuple_i64(tuple, 13).max(0) as u64,
        data_dir: tuple_string(tuple, 10),
        etag: String::new(),
        content_type: metadata
            .get("content-type")
            .cloned()
            .unwrap_or_else(|| "application/octet-stream".to_string()),
        user_metadata: metadata,
        erasure_index: erasure.index,
        erasure_m: erasure.data_blocks,
        erasure_n: erasure.parity_blocks,
    })
}

fn decode_minio_erasure_info(value: &Value) -> Result<MinioErasureInfo, String> {
    let map = value
        .as_map()
        .ok_or_else(|| "MinIO FileInfo.Erasure is not encoded as a map".to_string())?;
    Ok(MinioErasureInfo {
        data_blocks: map_i32(map, "data").unwrap_or(1),
        parity_blocks: map_i32(map, "parity").unwrap_or(0),
        index: map_i32(map, "index").unwrap_or(1),
    })
}

fn tuple_string(tuple: &[Value], index: usize) -> String {
    tuple
        .get(index)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn tuple_i64(tuple: &[Value], index: usize) -> i64 {
    tuple.get(index).and_then(Value::as_i64).unwrap_or_default()
}

fn map_i32(map: &[(Value, Value)], key: &str) -> Option<i32> {
    map.iter()
        .find(|(k, _)| k.as_str() == Some(key))
        .and_then(|(_, v)| v.as_i64())
        .map(|v| v as i32)
}

fn value_string_map(value: &Value) -> HashMap<String, String> {
    value
        .as_map()
        .map(|entries| {
            entries
                .iter()
                .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

struct MinioErasureInfo {
    data_blocks: i32,
    parity_blocks: i32,
    index: i32,
}

fn write_string_entry(out: &mut Vec<u8>, key: &str, value: &str) -> Result<(), String> {
    write_str(out, key).map_err(|err| err.to_string())?;
    write_str(out, value).map_err(|err| err.to_string())?;
    Ok(())
}

fn write_bytes_entry(out: &mut Vec<u8>, key: &str, value: &[u8]) -> Result<(), String> {
    write_str(out, key).map_err(|err| err.to_string())?;
    write_bin(out, value).map_err(|err| err.to_string())?;
    Ok(())
}

fn write_bool_entry(out: &mut Vec<u8>, key: &str, value: bool) -> Result<(), String> {
    write_str(out, key).map_err(|err| err.to_string())?;
    write_bool(out, value).map_err(|err| err.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_minio_file_info_tuple_minimal_fields() {
        let payload_value = Value::Array(vec![
            Value::from("bucket"),
            Value::from("object.txt"),
            Value::from("vid-1"),
            Value::from(true),
            Value::from(false),
            Value::from(""),
            Value::from(""),
            Value::from(""),
            Value::from(""),
            Value::from(false),
            Value::from("data-dir-1"),
            Value::from(false),
            Value::Nil,
            Value::from(123_i64),
            Value::from(0_u32),
            Value::from(0_u64),
            Value::Map(vec![
                (Value::from("content-type"), Value::from("text/plain")),
                (Value::from("x-amz-meta-k"), Value::from("v")),
            ]),
            Value::Array(vec![]),
            Value::Map(vec![
                (Value::from("data"), Value::from(2_i64)),
                (Value::from("parity"), Value::from(1_i64)),
                (Value::from("index"), Value::from(1_i64)),
            ]),
        ]);
        let mut payload = Vec::new();
        rmpv::encode::write_value(&mut payload, &payload_value).unwrap();

        let decoded = decode_minio_file_info(&payload).unwrap();
        assert_eq!(decoded.volume, "bucket");
        assert_eq!(decoded.name, "object.txt");
        assert_eq!(decoded.version_id, "vid-1");
        assert_eq!(decoded.size, 123);
        assert_eq!(decoded.data_dir, "data-dir-1");
        assert_eq!(decoded.content_type, "text/plain");
        assert_eq!(decoded.user_metadata.get("x-amz-meta-k").unwrap(), "v");
        assert_eq!(decoded.erasure_m, 2);
        assert_eq!(decoded.erasure_n, 1);
        assert_eq!(decoded.erasure_index, 1);
    }

    #[test]
    fn encode_minio_write_all_request_includes_expected_keys() {
        let encoded = encode_minio_write_all_request(&MinioWriteAllRequest {
            disk_id: "disk-1".to_string(),
            volume: "bucket".to_string(),
            file_path: "path/xl.meta".to_string(),
            buf: vec![1, 2, 3],
        })
        .unwrap();

        let decoded = rmpv::decode::read_value(&mut Cursor::new(&encoded)).unwrap();
        let map = decoded.as_map().unwrap();
        assert!(map.iter().any(|(k, v)| k.as_str() == Some("id") && v.as_str() == Some("disk-1")));
        assert!(map.iter().any(|(k, v)| k.as_str() == Some("b") && v.as_slice() == Some(&[1, 2, 3][..])));
    }
}
