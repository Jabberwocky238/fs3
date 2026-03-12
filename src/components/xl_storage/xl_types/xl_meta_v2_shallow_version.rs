use super::msgpack_compat::{MsgpackReader, MsgpackWriter};
use super::xl_meta_v2_version_header::XlMetaV2VersionHeader;

#[derive(Debug, Clone)]
pub struct XlMetaV2ShallowVersion {
    pub header: XlMetaV2VersionHeader,
    pub meta: Vec<u8>,
}

impl From<Vec<u8>> for XlMetaV2ShallowVersion {
    fn from(bytes: Vec<u8>) -> Self {
        let r = MsgpackReader::new(&bytes);
        Self {
            header: r.get_bytes("h").unwrap().into(),
            meta: r.get_bytes("m").unwrap_or_default(),
        }
    }
}

impl From<XlMetaV2ShallowVersion> for Vec<u8> {
    fn from(val: XlMetaV2ShallowVersion) -> Self {
        let mut w = MsgpackWriter::new();
        w.write_map_len(2);
        let header_bytes: Vec<u8> = val.header.into();
        w.write_bin_field("h", &header_bytes);
        w.write_bin_field("m", &val.meta);
        w.finish()
    }
}
