// 模块组织

mod types;
mod version_header;
mod object;
mod delete_marker;
mod version;
mod xl_meta;
mod shallow_version;
mod v1_object;
mod golang_map;

// 导出
pub use types::*;
pub use version_header::XlMetaV2VersionHeader;
pub use object::XlMetaV2Object;
pub use delete_marker::XlMetaV2DeleteMarker;
pub use version::XlMetaV2Version;
pub use xl_meta::XlMetaV2;
pub use shallow_version::XlMetaV2ShallowVersion;
pub use v1_object::*;
pub use golang_map::GoMapDecoder;
use std::error::Error;

pub trait XLMetaSerializer {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>>;
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> where Self: Sized;
}