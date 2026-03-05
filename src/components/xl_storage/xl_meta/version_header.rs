/// Version Header 结构体定义

use super::types::VersionType;
use super::version::XlMetaV2Version;
use super::XLMetaSerializer;
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct XlMetaV2VersionHeader {
    pub version_id: [u8; 16],
    pub mod_time: i64,
    pub signature: [u8; 4],
    pub version_type: VersionType,
    pub flags: u8,
    pub ec_n: i32,
    pub ec_m: i32,
}

impl From<&XlMetaV2Version> for XlMetaV2VersionHeader {
    fn from(ver: &XlMetaV2Version) -> Self {
        let (vid, mod_time, ec_n, ec_m) = match &ver.object_v2 {
            Some(obj) => (obj.version_id, obj.mod_time, obj.erasure_n, obj.erasure_m),
            None => match &ver.delete_marker {
                Some(dm) => (dm.version_id, dm.mod_time, 0, 0),
                None => ([0u8; 16], 0, 0, 0),
            }
        };

        Self {
            version_id: vid,
            mod_time,
            signature: [0u8; 4],
            version_type: ver.version_type,
            flags: 0,
            ec_n,
            ec_m,
        }
    }
}

impl XLMetaSerializer for XlMetaV2VersionHeader {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        rmp::encode::write_bin(&mut buf, &self.version_id)?;
        rmp::encode::write_sint(&mut buf, self.mod_time)?;
        rmp::encode::write_bin(&mut buf, &self.signature)?;
        rmp::encode::write_uint(&mut buf, self.version_type as u64)?;
        rmp::encode::write_uint(&mut buf, self.flags as u64)?;
        rmp::encode::write_uint(&mut buf, self.ec_n as u64)?;
        rmp::encode::write_uint(&mut buf, self.ec_m as u64)?;
        Ok(buf)
    }

    fn decode(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Self::decode_with_minor(data, 3)
    }
}

impl XlMetaV2VersionHeader {
    pub fn decode_with_minor(data: &[u8], minor: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let mut cursor = Cursor::new(data);

        let mut version_id = [0u8; 16];
        let vid_len = rmp::decode::read_bin_len(&mut cursor)?;
        if vid_len == 16 {
            let pos = cursor.position() as usize;
            version_id.copy_from_slice(&data[pos..pos + 16]);
            cursor.set_position((pos + 16) as u64);
        }

        let mod_time: i64 = rmp::decode::read_int(&mut cursor)?;

        let signature = if minor >= 2 {
            let sig_len = rmp::decode::read_bin_len(&mut cursor)?;
            let mut sig = [0u8; 4];
            if sig_len >= 4 {
                let pos = cursor.position() as usize;
                sig.copy_from_slice(&data[pos..pos + 4]);
            }
            cursor.set_position(cursor.position() + sig_len as u64);
            sig
        } else {
            [0u8; 4]
        };

        let vtype: u64 = rmp::decode::read_int(&mut cursor)?;
        let version_type = match vtype {
            1 => VersionType::Object,
            2 => VersionType::Delete,
            3 => VersionType::Legacy,
            _ => VersionType::Invalid,
        };

        let flags: u64 = rmp::decode::read_int(&mut cursor)?;

        let (ec_n, ec_m) = if minor >= 3 {
            let n: i64 = rmp::decode::read_int(&mut cursor)?;
            let m: i64 = rmp::decode::read_int(&mut cursor)?;
            (n as i32, m as i32)
        } else {
            (0, 0)
        };

        Ok(Self {
            version_id,
            mod_time,
            signature,
            version_type,
            flags: flags as u8,
            ec_n,
            ec_m,
        })
    }
}
