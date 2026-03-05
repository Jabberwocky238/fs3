mod xl_meta_types;
mod xl_meta_encode;
mod xl_meta_decode;

pub use xl_meta_types::*;
use xl_meta_encode::encode_xl_meta;
use xl_meta_decode::decode_xl_meta;

impl XlMetaV2 {
    pub fn encode(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        encode_xl_meta(self)
    }

    pub fn decode(buf: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        decode_xl_meta(buf)
    }
}
