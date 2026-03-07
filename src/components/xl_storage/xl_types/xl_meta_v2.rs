use super::msgpack_compat::{MsgpackReader, MsgpackWriter};
use super::xl_meta_v2_shallow_version::XlMetaV2ShallowVersion;
use super::xl_meta_v2_version_header::XlMetaV2VersionHeader;

pub type XlMetaInlineData = Vec<u8>;

#[derive(Debug, Clone)]
pub struct XlMetaV2 {
    pub versions: Vec<XlMetaV2ShallowVersion>,
    pub inline_data: Option<XlMetaInlineData>,
    pub meta_v: u8,
}

impl From<Vec<u8>> for XlMetaV2 {
    fn from(bytes: Vec<u8>) -> Self {
        let r = MsgpackReader::new(&bytes);
        Self {
            versions: vec![],
            inline_data: r.get_bytes("Data"),
            meta_v: r.get_u8("MetaV").unwrap_or(1),
        }
    }
}

impl From<XlMetaV2> for Vec<u8> {
    fn from(val: XlMetaV2) -> Self {
        let mut w = MsgpackWriter::new();
        w.write_map_len(3);
        w.write_array_field("Versions", val.versions.len() as u32, |w| {
            for v in &val.versions {
                w.write_map_len(2);
                w.write_str("h");
                w.write_map_len(7);
                w.write_bin_field("vid", &v.header.version_id);
                w.write_int32_field("mt", v.header.mod_time as i32);
                w.write_bin_field("sig", &v.header.signature);
                w.write_u8_field("vt", v.header.version_type);
                w.write_u8_field("f", v.header.flags);
                w.write_u8_field("n", v.header.ec_n);
                w.write_u8_field("m", v.header.ec_m);
                w.write_bin_field("m", &v.meta);
            }
        });
        w.write_bin_field("Data", val.inline_data.as_deref().unwrap_or(&[]));
        w.write_u8_field("MetaV", val.meta_v);
        w.finish()
    }
}
