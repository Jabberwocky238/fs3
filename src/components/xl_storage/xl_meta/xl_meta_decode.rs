use super::xl_meta_types::*;
use std::collections::HashMap;

pub fn decode_xl_meta(buf: &[u8]) -> Result<XlMetaV2, Box<dyn std::error::Error>> {
    if buf.len() < 8 || &buf[0..4] != &XL_HEADER {
        return Err("invalid xl.meta header".into());
    }

    let mut pos = 8;
    if buf[pos] != 0xc6 {
        return Err("invalid msgpack format".into());
    }
    pos += 1;
    let payload_size = u32::from_be_bytes([buf[pos], buf[pos+1], buf[pos+2], buf[pos+3]]) as usize;
    pos += 4;

    let payload = &buf[pos..pos+payload_size];
    let mut versions = Vec::new();
    let mut obj_size = 0i64;

    if let Some(id_pos) = payload.windows(3).position(|w| w == b"\xa2ID") {
        let id_start = id_pos + 3;
        if id_start + 18 < payload.len() && payload[id_start] == 0xc4 && payload[id_start+1] == 0x10 {
            let mut version_id = [0u8; 16];
            version_id.copy_from_slice(&payload[id_start+2..id_start+18]);

            if let Some(ddir_pos) = payload.windows(5).position(|w| w == b"\xa4DDir") {
                let ddir_start = ddir_pos + 5;
                if ddir_start + 18 < payload.len() && payload[ddir_start] == 0xc4 && payload[ddir_start+1] == 0x10 {
                    let mut data_dir = [0u8; 16];
                    data_dir.copy_from_slice(&payload[ddir_start+2..ddir_start+18]);

                    if let Some(size_pos) = payload.windows(5).position(|w| w == b"\xa4Size") {
                        obj_size = payload.get(size_pos + 5).copied().unwrap_or(0) as i64;
                    }

                    let obj = XlMetaV2Object {
                        version_id, data_dir, ec_algo: 1, ec_m: 1, ec_n: 0,
                        ec_bsize: 1048576, ec_index: 1, ec_dist: vec![1],
                        csum_algo: 1, part_nums: vec![1], part_etags: None,
                        part_sizes: vec![obj_size], part_asizes: vec![obj_size],
                        size: obj_size, mod_time: 0,
                        meta_sys: HashMap::new(), meta_user: HashMap::new(),
                    };

                    versions.push(XlMetaV2Version {
                        version_type: 1,
                        object_v2: Some(obj),
                    });
                }
            }
        }
    }

    let inline_data = if obj_size > 0 && buf.len() >= obj_size as usize {
        buf[buf.len() - obj_size as usize..].to_vec()
    } else {
        Vec::new()
    };

    Ok(XlMetaV2 { versions, inline_data })
}
