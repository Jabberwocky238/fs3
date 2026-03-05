use super::xl_meta_types::*;
use rmp::encode;

pub fn encode_xl_meta(meta: &XlMetaV2) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&XL_HEADER);
    buf.extend_from_slice(&XL_VERSION_MAJOR.to_le_bytes());
    buf.extend_from_slice(&XL_VERSION_MINOR.to_le_bytes());
    buf.push(0xc6);
    let size_offset = buf.len();
    buf.extend_from_slice(&[0u8; 4]);
    let data_offset = buf.len();

    encode::write_uint(&mut buf, 3)?;
    encode::write_uint(&mut buf, 3)?;
    encode::write_array_len(&mut buf, meta.versions.len() as u32)?;

    for ver in &meta.versions {
        if let Some(obj) = &ver.object_v2 {
            encode_header(&mut buf, obj, ver)?;
            encode_full_metadata(&mut buf, obj, ver)?;
        }
    }

    let payload_size = (buf.len() - data_offset) as u32;
    buf[size_offset..size_offset + 4].copy_from_slice(&payload_size.to_be_bytes());

    let crc = crc32fast::hash(&buf[data_offset..]);
    buf.push(0xce);
    buf.extend_from_slice(&crc.to_be_bytes());
    buf.extend_from_slice(&meta.inline_data);

    Ok(buf)
}

fn encode_header(buf: &mut Vec<u8>, obj: &XlMetaV2Object, ver: &XlMetaV2Version) -> Result<(), Box<dyn std::error::Error>> {
    let mut header = Vec::new();
    encode::write_bin(&mut header, &obj.version_id)?;
    encode::write_sint(&mut header, obj.mod_time)?;
    encode::write_bin(&mut header, &[0u8; 4])?;
    encode::write_uint(&mut header, ver.version_type as u64)?;
    encode::write_uint(&mut header, 6)?;
    encode::write_uint(&mut header, 0)?;
    encode::write_uint(&mut header, obj.ec_m as u64)?;
    encode::write_bin(buf, &header)?;
    Ok(())
}

fn encode_full_metadata(buf: &mut Vec<u8>, obj: &XlMetaV2Object, ver: &XlMetaV2Version) -> Result<(), Box<dyn std::error::Error>> {
    let mut full = Vec::new();
    full.push(0x83);

    encode::write_str(&mut full, "Type")?;
    encode::write_uint(&mut full, ver.version_type as u64)?;

    encode::write_str(&mut full, "V2Obj")?;
    full.push(0xde); full.extend_from_slice(&[0x00, 0x11]);

    encode_object_fields(&mut full, obj)?;
    encode::write_bin(buf, &full)?;
    Ok(())
}

fn encode_object_fields(buf: &mut Vec<u8>, obj: &XlMetaV2Object) -> Result<(), Box<dyn std::error::Error>> {
    encode::write_str(buf, "ID")?;
    encode::write_bin(buf, &obj.version_id)?;
    encode::write_str(buf, "DDir")?;
    encode::write_bin(buf, &obj.data_dir)?;
    encode::write_str(buf, "EcAlgo")?;
    encode::write_uint(buf, obj.ec_algo as u64)?;
    encode::write_str(buf, "EcM")?;
    encode::write_uint(buf, obj.ec_m as u64)?;
    encode::write_str(buf, "EcN")?;
    encode::write_uint(buf, obj.ec_n as u64)?;
    encode::write_str(buf, "EcBSize")?;
    encode::write_sint(buf, obj.ec_bsize)?;
    encode::write_str(buf, "EcIndex")?;
    encode::write_uint(buf, obj.ec_index as u64)?;
    encode::write_str(buf, "EcDist")?;
    encode::write_array_len(buf, obj.ec_dist.len() as u32)?;
    for &d in &obj.ec_dist { encode::write_uint(buf, d as u64)?; }
    encode::write_str(buf, "CSumAlgo")?;
    encode::write_uint(buf, obj.csum_algo as u64)?;
    encode::write_str(buf, "PartNums")?;
    encode::write_array_len(buf, obj.part_nums.len() as u32)?;
    for &n in &obj.part_nums { encode::write_sint(buf, n as i64)?; }
    encode::write_str(buf, "PartETags")?;
    if let Some(ref etags) = obj.part_etags {
        encode::write_array_len(buf, etags.len() as u32)?;
        for etag in etags { encode::write_str(buf, etag)?; }
    } else { buf.push(0xc0); }
    encode::write_str(buf, "PartSizes")?;
    encode::write_array_len(buf, obj.part_sizes.len() as u32)?;
    for &s in &obj.part_sizes { encode::write_sint(buf, s)?; }
    encode::write_str(buf, "PartASizes")?;
    encode::write_array_len(buf, obj.part_asizes.len() as u32)?;
    for &s in &obj.part_asizes { encode::write_sint(buf, s)?; }
    encode::write_str(buf, "Size")?;
    encode::write_sint(buf, obj.size)?;
    encode::write_str(buf, "MTime")?;
    encode::write_sint(buf, obj.mod_time)?;
    encode::write_str(buf, "MetaSys")?;
    encode::write_map_len(buf, obj.meta_sys.len() as u32)?;
    for (k, v) in &obj.meta_sys {
        encode::write_str(buf, k)?;
        encode::write_bin(buf, v)?;
    }
    encode::write_str(buf, "MetaUsr")?;
    encode::write_map_len(buf, obj.meta_user.len() as u32)?;
    for (k, v) in &obj.meta_user {
        encode::write_str(buf, k)?;
        encode::write_str(buf, v)?;
    }
    Ok(())
}
