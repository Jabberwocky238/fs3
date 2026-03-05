/// Shallow version - 对应 MinIO xlMetaV2ShallowVersion

use super::version_header::XlMetaV2VersionHeader;

#[derive(Debug, Clone)]
pub struct XlMetaV2ShallowVersion {
    pub header: XlMetaV2VersionHeader,
    pub meta: Vec<u8>,
}
