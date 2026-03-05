/// Version entry - 对应 MinIO xlMetaV2Version

use super::types::VersionType;
use super::object::XlMetaV2Object;
use super::delete_marker::XlMetaV2DeleteMarker;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV2Version {
    #[serde(rename = "Type")]
    pub version_type: VersionType,
    #[serde(rename = "V2Obj", skip_serializing_if = "Option::is_none")]
    pub object_v2: Option<XlMetaV2Object>,
    #[serde(rename = "DelObj", skip_serializing_if = "Option::is_none")]
    pub delete_marker: Option<XlMetaV2DeleteMarker>,
}

impl XlMetaV2Version {
    pub fn valid(&self) -> bool {
        match self.version_type {
            VersionType::Object => {
                self.object_v2.as_ref().map_or(false, |obj| obj.mod_time > 0)
            }
            VersionType::Delete => {
                self.delete_marker.as_ref().map_or(false, |dm| dm.mod_time > 0)
            }
            _ => false,
        }
    }
}
