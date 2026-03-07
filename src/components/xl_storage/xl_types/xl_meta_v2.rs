use super::xl_meta_v2_shallow_version::XlMetaV2ShallowVersion;

pub type XlMetaInlineData = Vec<u8>;

#[derive(Debug, Clone)]
pub struct XlMetaV2 {
    pub versions: Vec<XlMetaV2ShallowVersion>,
    pub data: Option<XlMetaInlineData>,
    pub meta_v: u8,
}
