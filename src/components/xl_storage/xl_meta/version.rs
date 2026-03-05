/// Version entry - 对应 MinIO xlMetaV2Version

use super::types::VersionType;
use super::object::XlMetaV2Object;
use super::delete_marker::XlMetaV2DeleteMarker;
use super::golang_map::GoMapDecoder;
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

impl From<&XlMetaV2Version> for super::golang_struct::GoBytes {
    fn from(ver: &XlMetaV2Version) -> Self {
        use super::golang_struct::{GoStructBuilder, GoBytes};
        let mut b = GoStructBuilder::new(3);

        b.field_u8("Type", ver.version_type as u8);

        if let Some(ref obj) = ver.object_v2 {
            let obj_bytes: GoBytes = obj.into();
            b.field_nested("V2Obj", obj_bytes.as_ref());
        }

        if let Some(ref _dm) = ver.delete_marker {
            b.field_nil("DelObj");
        }

        b.field_u8("v", 0);

        b.build()
    }
}

impl TryFrom<&[u8]> for XlMetaV2Version {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut decoder = GoMapDecoder::new(data);
        let len = decoder.read_map_len()?;

        let mut version_type = VersionType::Invalid;
        let mut object_v2 = None;
        let mut delete_marker = None;

        for _ in 0..len {
            let key = decoder.read_str()?;
            match key.as_str() {
                "Type" => {
                    version_type = VersionType::from_u8(decoder.read_int()? as u8);
                }
                "V2Obj" => {
                    decoder.skip_value()?;
                }
                "DelObj" => {
                    decoder.skip_value()?;
                }
                _ => {
                    decoder.skip_value()?;
                }
            }
        }

        Ok(Self {
            version_type,
            object_v2,
            delete_marker,
        })
    }
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
