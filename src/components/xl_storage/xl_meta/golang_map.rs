/// 手写msgpack解码器 - 兼容Go msgp格式
use std::collections::HashMap;

pub struct GoMapDecoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> GoMapDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_byte(&mut self) -> Result<u8, String> {
        if self.pos >= self.data.len() {
            return Err("EOF".into());
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    pub fn read_map_len(&mut self) -> Result<u32, String> {
        let marker = self.read_byte()?;
        match marker {
            0x80..=0x8f => Ok((marker & 0x0f) as u32),
            0xde => {
                let b1 = self.read_byte()? as u32;
                let b2 = self.read_byte()? as u32;
                Ok((b1 << 8) | b2)
            }
            0xdf => {
                let b1 = self.read_byte()? as u32;
                let b2 = self.read_byte()? as u32;
                let b3 = self.read_byte()? as u32;
                let b4 = self.read_byte()? as u32;
                Ok((b1 << 24) | (b2 << 16) | (b3 << 8) | b4)
            }
            _ => Err(format!("Invalid map marker: 0x{:02x}", marker)),
        }
    }

    pub fn read_str(&mut self) -> Result<String, String> {
        let len = self.read_str_len()?;
        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;
        String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())
    }

    fn read_str_len(&mut self) -> Result<usize, String> {
        let marker = self.read_byte()?;
        match marker {
            0xa0..=0xbf => Ok((marker & 0x1f) as usize),
            0xd9 => Ok(self.read_byte()? as usize),
            0xda => {
                let b1 = self.read_byte()? as usize;
                let b2 = self.read_byte()? as usize;
                Ok((b1 << 8) | b2)
            }
            0xdb => {
                let b1 = self.read_byte()? as usize;
                let b2 = self.read_byte()? as usize;
                let b3 = self.read_byte()? as usize;
                let b4 = self.read_byte()? as usize;
                Ok((b1 << 24) | (b2 << 16) | (b3 << 8) | b4)
            }
            _ => Err(format!("Invalid str marker: 0x{:02x}", marker)),
        }
    }

    pub fn read_int(&mut self) -> Result<i64, String> {
        let marker = self.read_byte()?;
        match marker {
            0x00..=0x7f => Ok(marker as i64),
            0xe0..=0xff => Ok((marker as i8) as i64),
            0xcc => Ok(self.read_byte()? as i64),
            0xcd => {
                let b1 = self.read_byte()? as i64;
                let b2 = self.read_byte()? as i64;
                Ok((b1 << 8) | b2)
            }
            0xce => {
                let b1 = self.read_byte()? as i64;
                let b2 = self.read_byte()? as i64;
                let b3 = self.read_byte()? as i64;
                let b4 = self.read_byte()? as i64;
                Ok((b1 << 24) | (b2 << 16) | (b3 << 8) | b4)
            }
            0xcf => {
                let mut val = 0i64;
                for _ in 0..8 {
                    val = (val << 8) | (self.read_byte()? as i64);
                }
                Ok(val)
            }
            0xd0 => Ok(self.read_byte()? as i8 as i64),
            0xd1 => {
                let b1 = self.read_byte()? as i16;
                let b2 = self.read_byte()? as i16;
                Ok(((b1 << 8) | b2) as i64)
            }
            0xd2 => {
                let mut val = 0i32;
                for _ in 0..4 {
                    val = (val << 8) | (self.read_byte()? as i32);
                }
                Ok(val as i64)
            }
            0xd3 => {
                let mut val = 0i64;
                for _ in 0..8 {
                    val = (val << 8) | (self.read_byte()? as i64);
                }
                Ok(val)
            }
            _ => Err(format!("Invalid int marker: 0x{:02x}", marker)),
        }
    }

    pub fn read_bytes(&mut self) -> Result<Vec<u8>, String> {
        let len = self.read_bin_len()?;
        let bytes = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        Ok(bytes)
    }

    fn read_bin_len(&mut self) -> Result<usize, String> {
        let marker = self.read_byte()?;
        match marker {
            0xc4 => Ok(self.read_byte()? as usize),
            0xc5 => {
                let b1 = self.read_byte()? as usize;
                let b2 = self.read_byte()? as usize;
                Ok((b1 << 8) | b2)
            }
            0xc6 => {
                let b1 = self.read_byte()? as usize;
                let b2 = self.read_byte()? as usize;
                let b3 = self.read_byte()? as usize;
                let b4 = self.read_byte()? as usize;
                Ok((b1 << 24) | (b2 << 16) | (b3 << 8) | b4)
            }
            _ => Err(format!("Invalid bin marker: 0x{:02x}", marker)),
        }
    }

    pub fn read_array_len(&mut self) -> Result<u32, String> {
        let marker = self.read_byte()?;
        match marker {
            0x90..=0x9f => Ok((marker & 0x0f) as u32),
            0xdc => {
                let b1 = self.read_byte()? as u32;
                let b2 = self.read_byte()? as u32;
                Ok((b1 << 8) | b2)
            }
            0xdd => {
                let b1 = self.read_byte()? as u32;
                let b2 = self.read_byte()? as u32;
                let b3 = self.read_byte()? as u32;
                let b4 = self.read_byte()? as u32;
                Ok((b1 << 24) | (b2 << 16) | (b3 << 8) | b4)
            }
            _ => Err(format!("Invalid array marker: 0x{:02x}", marker)),
        }
    }

    pub fn skip_value(&mut self) -> Result<(), String> {
        let marker = self.read_byte()?;
        match marker {
            0x00..=0x7f | 0xe0..=0xff => Ok(()),
            0xc0 | 0xc2 | 0xc3 => Ok(()),
            0xcc | 0xd0 => { self.pos += 1; Ok(()) }
            0xcd | 0xd1 => { self.pos += 2; Ok(()) }
            0xce | 0xd2 | 0xca => { self.pos += 4; Ok(()) }
            0xcf | 0xd3 | 0xcb => { self.pos += 8; Ok(()) }
            0xa0..=0xbf => {
                let len = (marker & 0x1f) as usize;
                self.pos += len;
                Ok(())
            }
            0xc4 | 0xd9 => {
                let len = self.read_byte()? as usize;
                self.pos += len;
                Ok(())
            }
            0xc5 | 0xda => {
                let b1 = self.read_byte()? as usize;
                let b2 = self.read_byte()? as usize;
                let len = (b1 << 8) | b2;
                self.pos += len;
                Ok(())
            }
            0xc6 | 0xdb => {
                let b1 = self.read_byte()? as usize;
                let b2 = self.read_byte()? as usize;
                let b3 = self.read_byte()? as usize;
                let b4 = self.read_byte()? as usize;
                let len = (b1 << 24) | (b2 << 16) | (b3 << 8) | b4;
                self.pos += len;
                Ok(())
            }
            0x90..=0x9f => {
                let len = (marker & 0x0f) as u32;
                for _ in 0..len {
                    self.skip_value()?;
                }
                Ok(())
            }
            0xdc => {
                let b1 = self.read_byte()? as u32;
                let b2 = self.read_byte()? as u32;
                let len = (b1 << 8) | b2;
                for _ in 0..len {
                    self.skip_value()?;
                }
                Ok(())
            }
            0xdd => {
                let b1 = self.read_byte()? as u32;
                let b2 = self.read_byte()? as u32;
                let b3 = self.read_byte()? as u32;
                let b4 = self.read_byte()? as u32;
                let len = (b1 << 24) | (b2 << 16) | (b3 << 8) | b4;
                for _ in 0..len {
                    self.skip_value()?;
                }
                Ok(())
            }
            0x80..=0x8f => {
                let len = (marker & 0x0f) as u32;
                for _ in 0..len {
                    self.skip_value()?;
                    self.skip_value()?;
                }
                Ok(())
            }
            0xde => {
                let b1 = self.read_byte()? as u32;
                let b2 = self.read_byte()? as u32;
                let len = (b1 << 8) | b2;
                for _ in 0..len {
                    self.skip_value()?;
                    self.skip_value()?;
                }
                Ok(())
            }
            0xdf => {
                let b1 = self.read_byte()? as u32;
                let b2 = self.read_byte()? as u32;
                let b3 = self.read_byte()? as u32;
                let b4 = self.read_byte()? as u32;
                let len = (b1 << 24) | (b2 << 16) | (b3 << 8) | b4;
                for _ in 0..len {
                    self.skip_value()?;
                    self.skip_value()?;
                }
                Ok(())
            }
            _ => Err(format!("Unknown marker: 0x{:02x}", marker)),
        }
    }
}
