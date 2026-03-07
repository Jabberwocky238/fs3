use super::xl_meta_v2_shallow_version::XlMetaV2ShallowVersion;
use super::msgpack_compat::{MsgpackReader, MsgpackWriter};

pub type XlMetaInlineData = Vec<u8>;

#[derive(Debug, Clone)]
pub struct XlMetaV2 {
    pub versions: Vec<XlMetaV2ShallowVersion>,
    pub inline_data: Option<XlMetaInlineData>,
    pub meta_v: u8,
}

impl XlMetaV2 {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let r = MsgpackReader::new(bytes);
        Ok(Self {
            versions: vec![],
            inline_data: r.get_bytes("Data"),
            meta_v: r.get_u8("MetaV").unwrap_or(1),
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        let mut w = MsgpackWriter::new();
        w.write_map_len(3);
        w.write_str("Versions");
        w.write_array(self.versions.len() as u32, |w| {
            for v in &self.versions {
                let bytes: Vec<u8> = v.clone().into();
                w.write_bin(&bytes);
            }
        });
        w.write_bin_field("Data", self.inline_data.as_deref().unwrap_or(&[]));
        w.write_u8_field("MetaV", self.meta_v);
        Ok(w.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::xl_storage::xl_types::xl_meta_v2_version_header::XlMetaV2VersionHeader;

    #[test]
    fn test_case_1_empty() {
        let expected = hex::decode("83a856657273696f6e7390a444617461c400a54d6574615601").unwrap();
        let obj = XlMetaV2 { versions: vec![], data: Some(vec![]), meta_v: 1 };
        assert_eq!(obj.encode().unwrap(), expected);
    }

    #[test]
    fn test_case_2_single_version() {
        let expected = hex::decode("83a856657273696f6e739182a16887a3766964c4100102030405060708090a0b0c0d0e0f10a26d74d2499602d2a3736967c404786c3220a2767401a16600a16e04a16d02a16dc403010203a444617461c402aabba54d6574615601").unwrap();
        let obj = XlMetaV2 {
            versions: vec![XlMetaV2ShallowVersion {
                header: XlMetaV2VersionHeader { version_id: [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16], mod_time: 1234567890, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 0, ec_n: 4, ec_m: 2 },
                meta: vec![0x01,0x02,0x03],
            }],
            data: Some(vec![0xaa,0xbb]),
            meta_v: 1,
        };
        assert_eq!(obj.encode().unwrap(), expected);
    }

    #[test]
    fn test_case_3_two_versions() {
        let expected = hex::decode("83a856657273696f6e739282a16887a3766964c410ffeeddccbbaa99887766554433221100a26d74d2499602d3a3736967c404786c3220a2767401a16601a16e08a16d04a16dc402112282a16887a3766964c410deadbeefcafebabe123456789abcdef0a26d74d2499602d4a3736967c404786c3220a2767402a16602a16e02a16d01a16dc403334455a444617461c403ccddeea54d6574615601").unwrap();
        let obj = XlMetaV2 {
            versions: vec![
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0xff,0xee,0xdd,0xcc,0xbb,0xaa,0x99,0x88,0x77,0x66,0x55,0x44,0x33,0x22,0x11,0x00], mod_time: 1234567891, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 1, ec_n: 8, ec_m: 4 }, meta: vec![0x11,0x22] },
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0xde,0xad,0xbe,0xef,0xca,0xfe,0xba,0xbe,0x12,0x34,0x56,0x78,0x9a,0xbc,0xde,0xf0], mod_time: 1234567892, signature: [0x78,0x6c,0x32,0x20], version_type: 2, flags: 2, ec_n: 2, ec_m: 1 }, meta: vec![0x33,0x44,0x55] },
            ],
            data: Some(vec![0xcc,0xdd,0xee]),
            meta_v: 1,
        };
        assert_eq!(obj.encode().unwrap(), expected);
    }

    #[test]
    fn test_case_4_three_versions() {
        let expected = hex::decode("83a856657273696f6e739382a16887a3766964c410123456789abcdef01122334455667788a26d74d2499602d5a3736967c404786c3220a2767401a16603a16e10a16d08a16dc402a1a282a16887a3766964c4108877665544332211f0debc9a78563412a26d74d2499602d6a3736967c404786c3220a2767401a16600a16e06a16d03a16dc403b1b2b382a16887a3766964c410aabbccddeeff00112233445566778899a26d74d2499602d7a3736967c404786c3220a2767402a16601a16e0ca16d06a16dc401c1a444617461c40401020304a54d6574615601").unwrap();
        let obj = XlMetaV2 {
            versions: vec![
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0x12,0x34,0x56,0x78,0x9a,0xbc,0xde,0xf0,0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88], mod_time: 1234567893, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 3, ec_n: 16, ec_m: 8 }, meta: vec![0xa1,0xa2] },
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0x88,0x77,0x66,0x55,0x44,0x33,0x22,0x11,0xf0,0xde,0xbc,0x9a,0x78,0x56,0x34,0x12], mod_time: 1234567894, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 0, ec_n: 6, ec_m: 3 }, meta: vec![0xb1,0xb2,0xb3] },
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0xaa,0xbb,0xcc,0xdd,0xee,0xff,0x00,0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88,0x99], mod_time: 1234567895, signature: [0x78,0x6c,0x32,0x20], version_type: 2, flags: 1, ec_n: 12, ec_m: 6 }, meta: vec![0xc1] },
            ],
            data: Some(vec![0x01,0x02,0x03,0x04]),
            meta_v: 1,
        };
        assert_eq!(obj.encode().unwrap(), expected);
    }

    #[test]
    fn test_case_5_large_data() {
        let expected = hex::decode("83a856657273696f6e739182a16887a3766964c41001010101010101010101010101010101a26d74d2499602d8a3736967c404786c3220a2767401a16600a16e01a16d01a16dc408ffeeddccbbaa9988a444617461c40a102030405060708090a0a54d6574615601").unwrap();
        let obj = XlMetaV2 {
            versions: vec![XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0x01;16], mod_time: 1234567896, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 0, ec_n: 1, ec_m: 1 }, meta: vec![0xff,0xee,0xdd,0xcc,0xbb,0xaa,0x99,0x88] }],
            data: Some(vec![0x10,0x20,0x30,0x40,0x50,0x60,0x70,0x80,0x90,0xa0]),
            meta_v: 1,
        };
        assert_eq!(obj.encode().unwrap(), expected);
    }

    #[test]
    fn test_case_6_four_versions() {
        let expected = hex::decode("83a856657273696f6e739482a16887a3766964c41000000000000000000000000000000001a26d74d2499602d9a3736967c404786c3220a2767401a16600a16e02a16d01a16dc4010182a16887a3766964c41000000000000000000000000000000002a26d74d2499602daa3736967c404786c3220a2767401a16601a16e04a16d02a16dc4010282a16887a3766964c41000000000000000000000000000000003a26d74d2499602dba3736967c404786c3220a2767402a16600a16e08a16d04a16dc4010382a16887a3766964c41000000000000000000000000000000004a26d74d2499602dca3736967c404786c3220a2767401a16602a16e10a16d08a16dc40104a444617461c403f0f1f2a54d6574615601").unwrap();
        let obj = XlMetaV2 {
            versions: vec![
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1], mod_time: 1234567897, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 0, ec_n: 2, ec_m: 1 }, meta: vec![0x01] },
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2], mod_time: 1234567898, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 1, ec_n: 4, ec_m: 2 }, meta: vec![0x02] },
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3], mod_time: 1234567899, signature: [0x78,0x6c,0x32,0x20], version_type: 2, flags: 0, ec_n: 8, ec_m: 4 }, meta: vec![0x03] },
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4], mod_time: 1234567900, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 2, ec_n: 16, ec_m: 8 }, meta: vec![0x04] },
            ],
            data: Some(vec![0xf0,0xf1,0xf2]),
            meta_v: 1,
        };
        assert_eq!(obj.encode().unwrap(), expected);
    }

    #[test]
    fn test_case_7_five_versions() {
        let expected = hex::decode("83a856657273696f6e739582a16887a3766964c410f0f1f2f3f4f5f6f7f8f9fafbfcfdfeffa26d74d2499602dda3736967c404786c3220a2767401a16600a16e01a16d01a16dc401a082a16887a3766964c410e0e1e2e3e4e5e6e7e8e9eaebecedeeefa26d74d2499602dea3736967c404786c3220a2767401a16601a16e02a16d01a16dc401b082a16887a3766964c410d0d1d2d3d4d5d6d7d8d9dadbdcdddedfa26d74d2499602dfa3736967c404786c3220a2767402a16600a16e04a16d02a16dc401c082a16887a3766964c410c0c1c2c3c4c5c6c7c8c9cacbcccdcecfa26d74d2499602e0a3736967c404786c3220a2767401a16602a16e08a16d04a16dc401d082a16887a3766964c410b0b1b2b3b4b5b6b7b8b9babbbcbdbebfa26d74d2499602e1a3736967c404786c3220a2767401a16603a16e10a16d08a16dc401e0a444617461c4051122334455a54d6574615601").unwrap();
        let obj = XlMetaV2 {
            versions: vec![
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0xf0,0xf1,0xf2,0xf3,0xf4,0xf5,0xf6,0xf7,0xf8,0xf9,0xfa,0xfb,0xfc,0xfd,0xfe,0xff], mod_time: 1234567901, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 0, ec_n: 1, ec_m: 1 }, meta: vec![0xa0] },
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0xe0,0xe1,0xe2,0xe3,0xe4,0xe5,0xe6,0xe7,0xe8,0xe9,0xea,0xeb,0xec,0xed,0xee,0xef], mod_time: 1234567902, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 1, ec_n: 2, ec_m: 1 }, meta: vec![0xb0] },
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0xd0,0xd1,0xd2,0xd3,0xd4,0xd5,0xd6,0xd7,0xd8,0xd9,0xda,0xdb,0xdc,0xdd,0xde,0xdf], mod_time: 1234567903, signature: [0x78,0x6c,0x32,0x20], version_type: 2, flags: 0, ec_n: 4, ec_m: 2 }, meta: vec![0xc0] },
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0xc0,0xc1,0xc2,0xc3,0xc4,0xc5,0xc6,0xc7,0xc8,0xc9,0xca,0xcb,0xcc,0xcd,0xce,0xcf], mod_time: 1234567904, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 2, ec_n: 8, ec_m: 4 }, meta: vec![0xd0] },
                XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0xb0,0xb1,0xb2,0xb3,0xb4,0xb5,0xb6,0xb7,0xb8,0xb9,0xba,0xbb,0xbc,0xbd,0xbe,0xbf], mod_time: 1234567905, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 3, ec_n: 16, ec_m: 8 }, meta: vec![0xe0] },
            ],
            data: Some(vec![0x11,0x22,0x33,0x44,0x55]),
            meta_v: 1,
        };
        assert_eq!(obj.encode().unwrap(), expected);
    }

    #[test]
    fn test_case_8_meta_v2() {
        let expected = hex::decode("83a856657273696f6e739182a16887a3766964c410a0a1a2a3a4a5a6a7a8a9aaabacadaeafa26d74d2499602e2a3736967c404786c3220a2767401a16600a16e0aa16d05a16dc403998877a444617461c404deadbeefa54d6574615602").unwrap();
        let obj = XlMetaV2 {
            versions: vec![XlMetaV2ShallowVersion { header: XlMetaV2VersionHeader { version_id: [0xa0,0xa1,0xa2,0xa3,0xa4,0xa5,0xa6,0xa7,0xa8,0xa9,0xaa,0xab,0xac,0xad,0xae,0xaf], mod_time: 1234567906, signature: [0x78,0x6c,0x32,0x20], version_type: 1, flags: 0, ec_n: 10, ec_m: 5 }, meta: vec![0x99,0x88,0x77] }],
            data: Some(vec![0xde,0xad,0xbe,0xef]),
            meta_v: 2,
        };
        assert_eq!(obj.encode().unwrap(), expected);
    }
}


