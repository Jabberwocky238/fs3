use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV2 {
    pub versions: Vec<XlMetaVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaVersion {
    pub version_id: String,
    pub data_dir: String,
    pub size: u64,
    pub mod_time: i64,
}
