use super::xl_meta_v2_version_header::XlMetaV2VersionHeader;

#[derive(Debug, Clone)]
pub struct XlMetaV2ShallowVersion {
    pub header: XlMetaV2VersionHeader,
    pub meta: Vec<u8>,
}
