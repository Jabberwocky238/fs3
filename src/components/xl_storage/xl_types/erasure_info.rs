use serde::{Deserialize, Serialize};

use super::checksum_info::ChecksumInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureInfo {
    pub algorithm: String,
    pub data: i32,
    pub parity: i32,
    #[serde(rename = "blockSize")]
    pub block_size: i64,
    pub index: i32,
    pub distribution: Vec<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<Vec<ChecksumInfo>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msgpack_go_compat() {
        let cases = vec![
            (hex::decode("86a4616c676fab72656564736f6c6f6d6f6ea16404a17002a26273d200a00000a16901a46469737496010203040506").unwrap(), 4, 2, 10485760),
            (hex::decode("87a4616c676fab72656564736f6c6f6d6f6ea16402a17001a26273d200500000a16902a46469737493010203a263739183a2706e01a16101a168c40101").unwrap(), 2, 1, 5242880),
            (hex::decode("87a4616c676fab72656564736f6c6f6d6f6ea16408a17004a26273d201400000a16905a4646973749c0102030405060708090a0b0ca263739383a2706e01a16101a168c402010283a2706e02a16103a168c402030483a2706e03a16104a168c4020506").unwrap(), 8, 4, 20971520),
        ];

        for (i, (bytes, data, parity, block_size)) in cases.into_iter().enumerate() {
            let obj: ErasureInfo = rmp_serde::from_slice(&bytes).unwrap();
            assert_eq!(obj.data, data, "Case {} data", i + 1);
            assert_eq!(obj.parity, parity, "Case {} parity", i + 1);
            assert_eq!(obj.block_size, block_size, "Case {} block_size", i + 1);
        }
    }
}
