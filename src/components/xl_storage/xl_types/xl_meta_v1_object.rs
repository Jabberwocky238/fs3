use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{erasure_info::ErasureInfo, object_part_info::ObjectPartInfo, stat_info::StatInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV1Object {
    pub version: String,
    pub format: String,
    pub stat: StatInfo,
    pub erasure: ErasureInfo,
    pub minio: MinioInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<Vec<ObjectPartInfo>>,
    #[serde(rename = "versionId", skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,
    #[serde(rename = "dataDir", skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinioInfo {
    pub release: String,
}
