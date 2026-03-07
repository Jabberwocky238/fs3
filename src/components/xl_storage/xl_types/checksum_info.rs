use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u32)]
pub enum BitrotAlgorithm {
    SHA256 = 1,
    HighwayHash256 = 2,
    HighwayHash256S = 3,
    BLAKE2b512 = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumInfo {
    #[serde(rename = "partNumber")]
    pub part_number: i32,
    pub algorithm: BitrotAlgorithm,
    pub hash: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msgpack_go_compat() {
        let cases = vec![
            (hex::decode("83a2706e01a16101a168c403010203").unwrap(), 1, BitrotAlgorithm::SHA256, vec![0x01, 0x02, 0x03]),
            (hex::decode("83a2706e02a16102a168c404aabbccdd").unwrap(), 2, BitrotAlgorithm::HighwayHash256, vec![0xaa, 0xbb, 0xcc, 0xdd]),
        ];

        for (i, (bytes, pn, algo, hash)) in cases.into_iter().enumerate() {
            let obj: ChecksumInfo = rmp_serde::from_slice(&bytes).unwrap();
            assert_eq!(obj.part_number, pn, "Case {} part_number", i + 1);
            assert_eq!(obj.hash, hash, "Case {} hash", i + 1);
        }
    }
}
