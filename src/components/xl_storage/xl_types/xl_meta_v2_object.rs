use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::msgpack_compat::{MsgpackReader, MsgpackWriter};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum ErasureAlgo {
    #[serde(rename = "0")]
    Invalid = 0,
    #[serde(rename = "1")]
    ReedSolomon = 1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum ChecksumAlgo {
    #[serde(rename = "0")]
    Invalid = 0,
    #[serde(rename = "1")]
    HighwayHash = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XlMetaV2Object {
    pub version_id: [u8; 16],
    pub data_dir: [u8; 16],
    pub erasure_algorithm: ErasureAlgo,
    pub erasure_m: i32,
    pub erasure_n: i32,
    pub erasure_block_size: i64,
    pub erasure_index: i32,
    pub erasure_dist: Vec<u8>,
    pub bitrot_checksum_algo: ChecksumAlgo,
    pub part_numbers: Vec<i32>,
    pub part_etags: Vec<String>,
    pub part_sizes: Vec<i64>,
    pub part_actual_sizes: Option<Vec<i64>>,
    pub part_indices: Option<Vec<Vec<u8>>>,
    pub size: i64,
    pub mod_time: i64,
    pub meta_sys: Option<HashMap<String, Vec<u8>>>,
    pub meta_user: Option<HashMap<String, String>>,
}

impl From<Vec<u8>> for XlMetaV2Object {
    fn from(bytes: Vec<u8>) -> Self {
        let r = MsgpackReader::new(&bytes);
        Self {
            version_id: r.get_bytes("ID").unwrap().try_into().unwrap(),
            data_dir: r.get_bytes("DDir").unwrap().try_into().unwrap(),
            erasure_algorithm: if r.get_u8("EcAlgo").unwrap() == 1 { ErasureAlgo::ReedSolomon } else { ErasureAlgo::Invalid },
            erasure_m: r.get_i64("EcM").unwrap() as i32,
            erasure_n: r.get_i64("EcN").unwrap() as i32,
            erasure_block_size: r.get_i64("EcBSize").unwrap(),
            erasure_index: r.get_i64("EcIndex").unwrap() as i32,
            erasure_dist: r.get_u8_array("EcDist").unwrap(),
            bitrot_checksum_algo: if r.get_u8("CSumAlgo").unwrap() == 1 { ChecksumAlgo::HighwayHash } else { ChecksumAlgo::Invalid },
            part_numbers: r.get_int_array("PartNums").unwrap(),
            part_etags: r.get_str_array("PartETags").unwrap(),
            part_sizes: r.get_i64_array("PartSizes").unwrap(),
            part_actual_sizes: r.get_i64_array("PartASizes"),
            part_indices: None,
            size: r.get_i64("Size").unwrap(),
            mod_time: r.get_i64("MTime").unwrap(),
            meta_sys: r.get_bytes_map("MetaSys").map(|v| v.into_iter().collect()),
            meta_user: r.get_map("MetaUsr").map(|v| v.into_iter().collect()),
        }
    }
}

impl From<XlMetaV2Object> for Vec<u8> {
    fn from(val: XlMetaV2Object) -> Self {
        let mut w = MsgpackWriter::new();

        let field_count = 17 + val.part_indices.is_some() as u32;
        w.write_map_len(field_count);

        w.write_bytes_field("ID", &val.version_id);
        w.write_bytes_field("DDir", &val.data_dir);
        w.write_u8_field("EcAlgo", val.erasure_algorithm as u8);
        w.write_int_field("EcM", val.erasure_m as i64);
        w.write_int_field("EcN", val.erasure_n as i64);
        w.write_int_field("EcBSize", val.erasure_block_size);
        w.write_int_field("EcIndex", val.erasure_index as i64);
        w.write_array_field("EcDist", val.erasure_dist.len() as u32, |w| w.write_u8_array(&val.erasure_dist));
        w.write_u8_field("CSumAlgo", val.bitrot_checksum_algo as u8);
        w.write_array_field("PartNums", val.part_numbers.len() as u32, |w| w.write_int_array(&val.part_numbers));

        w.write_str("PartETags");
        if val.part_etags.is_empty() {
            w.write_nil();
        } else {
            w.write_array(val.part_etags.len() as u32, |w| w.write_str_array(&val.part_etags));
        }

        w.write_array_field("PartSizes", val.part_sizes.len() as u32, |w| w.write_i64_array(&val.part_sizes));

        w.write_str("PartASizes");
        if let Some(ref sizes) = val.part_actual_sizes {
            w.write_array(sizes.len() as u32, |w| w.write_i64_array(sizes));
        } else {
            w.write_nil();
        }

        if let Some(ref indices) = val.part_indices {
            w.write_array_field("PartIdx", indices.len() as u32, |w| {
                for idx in indices {
                    w.write_bin(idx);
                }
            });
        }

        w.write_int16_field("Size", val.size);
        w.write_int32_field("MTime", val.mod_time as i32);

        w.write_str("MetaSys");
        if let Some(ref meta) = val.meta_sys {
            w.write_map(meta.len() as u32, |w| {
                for (k, v) in meta {
                    w.write_bin_field(k, v);
                }
            });
        } else {
            w.write_nil();
        }

        w.write_str("MetaUsr");
        if let Some(ref meta) = val.meta_user {
            w.write_map(meta.len() as u32, |w| {
                for (k, v) in meta {
                    w.write_str_field(k, v);
                }
            });
        } else {
            w.write_nil();
        }

        w.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_1() {
        let obj = XlMetaV2Object {
            version_id: [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16],
            data_dir: [0xaa,0xbb,0xcc,0xdd,0xee,0xff,0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88,0x99,0x00],
            erasure_algorithm: ErasureAlgo::Invalid,
            erasure_m: 0,
            erasure_n: 0,
            erasure_block_size: 0,
            erasure_index: 0,
            erasure_dist: vec![],
            bitrot_checksum_algo: ChecksumAlgo::Invalid,
            part_numbers: vec![],
            part_etags: vec![],
            part_sizes: vec![],
            part_actual_sizes: None,
            part_indices: None,
            size: 1024,
            mod_time: 1234567890,
            meta_sys: None,
            meta_user: None,
        };
        let rust_bytes: Vec<u8> = obj.clone().into();
        let expected = hex::decode("de0011a24944c4100102030405060708090a0b0c0d0e0f10a444446972c410aabbccddeeff11223344556677889900a64563416c676f00a345634d00a345634e00a745634253697a6500a74563496e64657800a645634469737490a84353756d416c676f00a8506172744e756d7390a9506172744554616773c0a95061727453697a657390aa506172744153697a6573c0a453697a65d10400a54d54696d65d2499602d2a74d657461537973c0a74d657461557372c0").unwrap();
        assert_eq!(rust_bytes, expected);
        let decoded: XlMetaV2Object = expected.into();
        assert_eq!(decoded.version_id, obj.version_id);
        assert_eq!(decoded.size, obj.size);
    }

    #[test]
    fn test_case_2() {
        let obj = XlMetaV2Object {
            version_id: [0xff,0xee,0xdd,0xcc,0xbb,0xaa,0x99,0x88,0x77,0x66,0x55,0x44,0x33,0x22,0x11,0x00],
            data_dir: [0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88,0x99,0xaa,0xbb,0xcc,0xdd,0xee,0xff,0x00],
            erasure_algorithm: ErasureAlgo::Invalid,
            erasure_m: 0,
            erasure_n: 0,
            erasure_block_size: 0,
            erasure_index: 0,
            erasure_dist: vec![],
            bitrot_checksum_algo: ChecksumAlgo::Invalid,
            part_numbers: vec![],
            part_etags: vec![],
            part_sizes: vec![],
            part_actual_sizes: None,
            part_indices: None,
            size: 2048,
            mod_time: 1234567891,
            meta_sys: None,
            meta_user: None,
        };
        let rust_bytes: Vec<u8> = obj.clone().into();
        let expected = hex::decode("de0011a24944c410ffeeddccbbaa99887766554433221100a444446972c410112233445566778899aabbccddeeff00a64563416c676f00a345634d00a345634e00a745634253697a6500a74563496e64657800a645634469737490a84353756d416c676f00a8506172744e756d7390a9506172744554616773c0a95061727453697a657390aa506172744153697a6573c0a453697a65d10800a54d54696d65d2499602d3a74d657461537973c0a74d657461557372c0").unwrap();
        assert_eq!(rust_bytes, expected);
        let decoded: XlMetaV2Object = expected.into();
        assert_eq!(decoded.size, 2048);
    }

    #[test]
    fn test_case_3() {
        let obj = XlMetaV2Object {
            version_id: [0xde,0xad,0xbe,0xef,0xca,0xfe,0xba,0xbe,0x12,0x34,0x56,0x78,0x9a,0xbc,0xde,0xf0],
            data_dir: [0xf0,0xde,0xbc,0x9a,0x78,0x56,0x34,0x12,0xbe,0xba,0xfe,0xca,0xef,0xbe,0xad,0xde],
            erasure_algorithm: ErasureAlgo::Invalid,
            erasure_m: 0,
            erasure_n: 0,
            erasure_block_size: 0,
            erasure_index: 0,
            erasure_dist: vec![],
            bitrot_checksum_algo: ChecksumAlgo::Invalid,
            part_numbers: vec![],
            part_etags: vec![],
            part_sizes: vec![],
            part_actual_sizes: None,
            part_indices: None,
            size: 4096,
            mod_time: 1234567892,
            meta_sys: None,
            meta_user: None,
        };
        let rust_bytes: Vec<u8> = obj.clone().into();
        let expected = hex::decode("de0011a24944c410deadbeefcafebabe123456789abcdef0a444446972c410f0debc9a78563412bebafecaefbeaddea64563416c676f00a345634d00a345634e00a745634253697a6500a74563496e64657800a645634469737490a84353756d416c676f00a8506172744e756d7390a9506172744554616773c0a95061727453697a657390aa506172744153697a6573c0a453697a65d11000a54d54696d65d2499602d4a74d657461537973c0a74d657461557372c0").unwrap();
        assert_eq!(rust_bytes, expected);
        let decoded: XlMetaV2Object = expected.into();
        assert_eq!(decoded.size, 4096);
    }

    #[test]
    fn test_case_4() {
        let obj = XlMetaV2Object {
            version_id: [0x12,0x34,0x56,0x78,0x9a,0xbc,0xde,0xf0,0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88],
            data_dir: [0x88,0x77,0x66,0x55,0x44,0x33,0x22,0x11,0xf0,0xde,0xbc,0x9a,0x78,0x56,0x34,0x12],
            erasure_algorithm: ErasureAlgo::ReedSolomon,
            erasure_m: 10,
            erasure_n: 5,
            erasure_block_size: 4194304,
            erasure_index: 3,
            erasure_dist: vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14],
            bitrot_checksum_algo: ChecksumAlgo::HighwayHash,
            part_numbers: vec![1,2,3],
            part_etags: vec!["part1".to_string(),"part2".to_string(),"part3".to_string()],
            part_sizes: vec![1024,2048,4096],
            part_actual_sizes: Some(vec![1000,2000,4000]),
            part_indices: Some(vec![vec![1,2],vec![3,4],vec![5,6]]),
            size: 7168,
            mod_time: 1234567893,
            meta_sys: Some([("sys1".to_string(),vec![0x01,0x02])].into()),
            meta_user: None,
        };
        let rust_bytes: Vec<u8> = obj.into();
        let expected = hex::decode("de0012a24944c410123456789abcdef01122334455667788a444446972c4108877665544332211f0debc9a78563412a64563416c676f01a345634d0aa345634e05a745634253697a65d200400000a74563496e64657803a64563446973749f000102030405060708090a0b0c0d0ea84353756d416c676f01a8506172744e756d7393010203a950617274455461677393a57061727431a57061727432a57061727433a95061727453697a657393d10400d10800d11000aa506172744153697a657393d103e8d107d0d10fa0a75061727449647893c4020102c4020304c4020506a453697a65d11c00a54d54696d65d2499602d5a74d65746153797381a473797331c4020102a74d657461557372c0").unwrap();
        assert_eq!(rust_bytes, expected);
    }

    #[test]
    fn test_case_5() {
        let obj = XlMetaV2Object {
            version_id: [0xaa,0xbb,0xcc,0xdd,0xee,0xff,0x00,0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88,0x99],
            data_dir: [0x99,0x88,0x77,0x66,0x55,0x44,0x33,0x22,0x11,0x00,0xff,0xee,0xdd,0xcc,0xbb,0xaa],
            erasure_algorithm: ErasureAlgo::ReedSolomon,
            erasure_m: 12,
            erasure_n: 6,
            erasure_block_size: 8388608,
            erasure_index: 4,
            erasure_dist: vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17],
            bitrot_checksum_algo: ChecksumAlgo::HighwayHash,
            part_numbers: vec![],
            part_etags: vec![],
            part_sizes: vec![],
            part_actual_sizes: None,
            part_indices: None,
            size: 0,
            mod_time: 1234567894,
            meta_sys: None,
            meta_user: None,
        };
        let rust_bytes: Vec<u8> = obj.into();
        let expected = hex::decode("de0011a24944c410aabbccddeeff00112233445566778899a444446972c41099887766554433221100ffeeddccbbaaa64563416c676f01a345634d0ca345634e06a745634253697a65d200800000a74563496e64657804a6456344697374dc0012000102030405060708090a0b0c0d0e0f1011a84353756d416c676f01a8506172744e756d7390a9506172744554616773c0a95061727453697a657390aa506172744153697a6573c0a453697a6500a54d54696d65d2499602d6a74d657461537973c0a74d657461557372c0").unwrap();
        assert_eq!(rust_bytes, expected);
    }

    #[test]
    fn test_case_6() {
        let obj = XlMetaV2Object {
            version_id: [0x01;16],
            data_dir: [0x02;16],
            erasure_algorithm: ErasureAlgo::ReedSolomon,
            erasure_m: 2,
            erasure_n: 1,
            erasure_block_size: 262144,
            erasure_index: 0,
            erasure_dist: vec![0,1,2],
            bitrot_checksum_algo: ChecksumAlgo::HighwayHash,
            part_numbers: vec![1,2,3,4,5],
            part_etags: vec!["e1".to_string(),"e2".to_string(),"e3".to_string(),"e4".to_string(),"e5".to_string()],
            part_sizes: vec![100,200,300,400,500],
            part_actual_sizes: None,
            part_indices: None,
            size: 1500,
            mod_time: 1234567895,
            meta_sys: None,
            meta_user: Some([("content-type".to_string(),"application/octet-stream".to_string()),("x-custom".to_string(),"test".to_string())].into()),
        };
        let rust_bytes: Vec<u8> = obj.clone().into();
        let expected = hex::decode("de0011a24944c41001010101010101010101010101010101a444446972c41002020202020202020202020202020202a64563416c676f01a345634d02a345634e01a745634253697a65d200040000a74563496e64657800a645634469737493000102a84353756d416c676f01a8506172744e756d73950102030405a950617274455461677395a26531a26532a26533a26534a26535a95061727453697a65739564d100c8d1012cd10190d101f4aa506172744153697a6573c0a453697a65d105dca54d54696d65d2499602d7a74d657461537973c0a74d65746155737282ac636f6e74656e742d74797065b86170706c69636174696f6e2f6f637465742d73747265616da8782d637573746f6da474657374").unwrap();
        assert_eq!(rust_bytes, expected);
        let decoded: XlMetaV2Object = expected.into();
        assert_eq!(decoded.version_id, obj.version_id);
        assert_eq!(decoded.erasure_m, obj.erasure_m);
        assert_eq!(decoded.part_numbers, obj.part_numbers);
    }

    #[test]
    fn test_case_7() {
        let obj = XlMetaV2Object {
            version_id: [0xde,0xad,0xbe,0xef,0xca,0xfe,0xba,0xbe,0x12,0x34,0x56,0x78,0x9a,0xbc,0xde,0xf0],
            data_dir: [0xf0,0xde,0xbc,0x9a,0x78,0x56,0x34,0x12,0xbe,0xba,0xfe,0xca,0xef,0xbe,0xad,0xde],
            erasure_algorithm: ErasureAlgo::ReedSolomon,
            erasure_m: 16,
            erasure_n: 8,
            erasure_block_size: 16777216,
            erasure_index: 7,
            erasure_dist: vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23],
            bitrot_checksum_algo: ChecksumAlgo::HighwayHash,
            part_numbers: vec![1,2,3,4],
            part_etags: vec!["large1".to_string(),"large2".to_string(),"large3".to_string(),"large4".to_string()],
            part_sizes: vec![10485760,10485760,10485760,10485760],
            part_actual_sizes: Some(vec![10000000,10000000,10000000,10000000]),
            part_indices: Some(vec![vec![0x01],vec![0x02],vec![0x03],vec![0x04]]),
            size: 41943040,
            mod_time: 1234567896,
            meta_sys: Some([("encryption".to_string(),vec![0xaa,0xbb,0xcc]),("compression".to_string(),vec![0x01])].into()),
            meta_user: Some([("content-type".to_string(),"video/mp4".to_string()),("x-amz-meta-custom".to_string(),"large-file".to_string())].into()),
        };
        let rust_bytes: Vec<u8> = obj.clone().into();
        let expected = hex::decode("de0012a24944c410deadbeefcafebabe123456789abcdef0a444446972c410f0debc9a78563412bebafecaefbeaddea64563416c676f01a345634d10a345634e08a745634253697a65d201000000a74563496e64657807a6456344697374dc0018000102030405060708090a0b0c0d0e0f1011121314151617a84353756d416c676f01a8506172744e756d739401020304a950617274455461677394a66c6172676531a66c6172676532a66c6172676533a66c6172676534a95061727453697a657394d200a00000d200a00000d200a00000d200a00000aa506172744153697a657394d200989680d200989680d200989680d200989680a75061727449647894c40101c40102c40103c40104a453697a65d202800000a54d54696d65d2499602d8a74d65746153797382aa656e6372797074696f6ec403aabbccab636f6d7072657373696f6ec40101a74d65746155737282ac636f6e74656e742d74797065a9766964656f2f6d7034b1782d616d7a2d6d6574612d637573746f6daa6c617267652d66696c65").unwrap();
        assert_eq!(rust_bytes, expected);
        let decoded: XlMetaV2Object = expected.into();
        assert_eq!(decoded.erasure_m, 16);
        assert_eq!(decoded.part_numbers, vec![1,2,3,4]);
    }

    #[test]
    fn test_case_8() {
        let obj = XlMetaV2Object {
            version_id: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
            data_dir: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            erasure_algorithm: ErasureAlgo::ReedSolomon,
            erasure_m: 1,
            erasure_n: 1,
            erasure_block_size: 65536,
            erasure_index: 0,
            erasure_dist: vec![0,1],
            bitrot_checksum_algo: ChecksumAlgo::HighwayHash,
            part_numbers: vec![1,2,3,4,5,6,7,8,9,10],
            part_etags: vec!["p1".to_string(),"p2".to_string(),"p3".to_string(),"p4".to_string(),"p5".to_string(),"p6".to_string(),"p7".to_string(),"p8".to_string(),"p9".to_string(),"p10".to_string()],
            part_sizes: vec![50,50,50,50,50,50,50,50,50,50],
            part_actual_sizes: Some(vec![48,48,48,48,48,48,48,48,48,48]),
            part_indices: Some(vec![vec![0],vec![1],vec![2],vec![3],vec![4],vec![5],vec![6],vec![7],vec![8],vec![9]]),
            size: 500,
            mod_time: 1234567897,
            meta_sys: Some([("key".to_string(),vec![0xff])].into()),
            meta_user: Some([("x-test".to_string(),"multipart".to_string())].into()),
        };
        let rust_bytes: Vec<u8> = obj.clone().into();
        let expected = hex::decode("de0012a24944c41000000000000000000000000000000001a444446972c41000000000000000000000000000000002a64563416c676f01a345634d01a345634e01a745634253697a65d200010000a74563496e64657800a6456344697374920001a84353756d416c676f01a8506172744e756d739a0102030405060708090aa95061727445546167739aa27031a27032a27033a27034a27035a27036a27037a27038a27039a3703130a95061727453697a65739a32323232323232323232aa506172744153697a65739a30303030303030303030a7506172744964789ac40100c40101c40102c40103c40104c40105c40106c40107c40108c40109a453697a65d101f4a54d54696d65d2499602d9a74d65746153797381a36b6579c401ffa74d65746155737281a6782d74657374a96d756c746970617274").unwrap();
        assert_eq!(rust_bytes, expected);
        let decoded: XlMetaV2Object = expected.into();
        assert_eq!(decoded.part_numbers.len(), 10);
    }
}
