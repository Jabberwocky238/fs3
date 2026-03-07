mod xl_storage;

pub use xl_storage::check_path_length;
use std::error::Error;

pub trait XLMetaSerializer {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>>;
    fn decode(buf: &[u8]) -> Result<Self, Box<dyn Error>> where Self: Sized;
}