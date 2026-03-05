/// Delete marker - 对应 MinIO xlMetaV2DeleteMarker

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV2DeleteMarker {
    #[serde(rename = "ID")]
    pub version_id: [u8; 16],
    #[serde(rename = "MTime")]
    pub mod_time: i64,
    #[serde(rename = "MetaSys", skip_serializing_if = "HashMap::is_empty", default)]
    pub meta_sys: HashMap<String, Vec<u8>>,
}
