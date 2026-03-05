/// XlMetaV2 主结构
use super::types::*;
use super::version::XlMetaV2Version;
use super::version_header::XlMetaV2VersionHeader;
use super::XLMetaSerializer;

#[derive(Debug, Clone)]
pub struct XlMetaV2 {
    pub versions: Vec<XlMetaV2Version>,
    pub inline_data: Vec<u8>,
}

impl XLMetaSerializer for XlMetaV2 {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&XL_HEADER);
        buf.extend_from_slice(&XL_VERSION_MAJOR.to_le_bytes());
        buf.extend_from_slice(&XL_VERSION_MINOR.to_le_bytes());

        let payload = self.encode_payload()?;
        buf.push(0xc6);
        buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        buf.extend_from_slice(&payload);

        let crc = crc32fast::hash(&payload);
        buf.push(0xce);
        buf.extend_from_slice(&crc.to_be_bytes());
        buf.extend_from_slice(&self.inline_data);

        Ok(buf)
    }

    fn decode(buf: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if buf.len() < 8 || &buf[0..4] != &XL_HEADER { return Err("invalid header".into()); }

        let minor = u16::from_le_bytes([buf[6], buf[7]]);
        let mut pos = 9;
        let payload_size = u32::from_be_bytes([buf[pos], buf[pos+1], buf[pos+2], buf[pos+3]]) as usize;
        pos += 4;

        let versions = Self::decode_payload(&buf[pos..pos+payload_size], minor)?;
        pos += payload_size + 5;

        Ok(Self {
            versions,
            inline_data: if pos < buf.len() { buf[pos..].to_vec() } else { Vec::new() }
        })
    }
}

impl XlMetaV2 {
    fn encode_payload(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut p = Vec::new();
        rmp::encode::write_uint(&mut p, 3)?;
        rmp::encode::write_uint(&mut p, 3)?;
        rmp::encode::write_array_len(&mut p, self.versions.len() as u32)?;

        for ver in &self.versions {
            let h = XlMetaV2VersionHeader::from(ver);
            rmp::encode::write_bin(&mut p, &h.encode()?)?;
            rmp::encode::write_bin(&mut p, &rmp_serde::to_vec_named(ver)?)?;
        }
        Ok(p)
    }

    fn decode_payload(payload: &[u8], _minor: u16) -> Result<Vec<XlMetaV2Version>, Box<dyn std::error::Error>> {
        use std::io::Cursor;
        let mut c = Cursor::new(payload);
        let _: i64 = rmp::decode::read_int(&mut c)?;
        let _: i64 = rmp::decode::read_int(&mut c)?;

        let count = rmp::decode::read_array_len(&mut c)? as usize;
        let mut versions = Vec::with_capacity(count);

        for _ in 0..count {
            let hlen = rmp::decode::read_bin_len(&mut c)?;
            c.set_position(c.position() + hlen as u64);

            let mlen = rmp::decode::read_bin_len(&mut c)?;
            let pos = c.position() as usize;
            versions.push(rmp_serde::from_slice(&payload[pos..pos + mlen as usize])?);
            c.set_position((pos + mlen as usize) as u64);
        }
        Ok(versions)
    }
}
