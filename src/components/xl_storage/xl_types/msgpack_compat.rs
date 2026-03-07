use rmp::encode::*;
use rmpv::{decode::read_value, Value};
use std::io::Cursor;

pub struct MsgpackWriter {
    buf: Vec<u8>,
}

impl MsgpackWriter {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn write_map_len(&mut self, len: u32) {
        rmp::encode::write_map_len(&mut self.buf, len).unwrap();
    }

    pub fn write_str_field(&mut self, key: &str, val: &str) {
        write_str(&mut self.buf, key).unwrap();
        write_str(&mut self.buf, val).unwrap();
    }

    pub fn write_int64_field(&mut self, key: &str, val: i64) {
        write_str(&mut self.buf, key).unwrap();
        write_sint(&mut self.buf, val).unwrap();
    }

    pub fn write_int16_field(&mut self, key: &str, val: i16) {
        write_str(&mut self.buf, key).unwrap();
        if val >= 0 && val <= 127 {
            self.buf.push(val as u8);
        } else if val >= -32 && val < 0 {
            self.buf.push((val as i8) as u8);
        } else {
            self.buf.push(0xd1);
            self.buf.extend_from_slice(&(val as i16).to_be_bytes());
        }
    }

    pub fn write_int32_field(&mut self, key: &str, val: i32) {
        write_str(&mut self.buf, key).unwrap();
        self.buf.push(0xd2);
        self.buf.extend_from_slice(&val.to_be_bytes());
    }

    pub fn write_str(&mut self, s: &str) {
        write_str(&mut self.buf, s).unwrap();
    }

    pub fn write_nil(&mut self) {
        self.buf.push(0xc0);
    }

    pub fn write_array<F>(&mut self, len: u32, f: F)
    where
        F: FnOnce(&mut Self),
    {
        write_array_len(&mut self.buf, len).unwrap();
        f(self);
    }

    pub fn write_map<F>(&mut self, len: u32, f: F)
    where
        F: FnOnce(&mut Self),
    {
        write_map_len(&mut self.buf, len).unwrap();
        f(self);
    }

    pub fn write_u8_field(&mut self, key: &str, val: u8) {
        write_str(&mut self.buf, key).unwrap();
        write_uint(&mut self.buf, val as u64).unwrap();
    }

    pub fn write_bytes_field(&mut self, key: &str, data: &[u8]) {
        write_str(&mut self.buf, key).unwrap();
        write_bin(&mut self.buf, data).unwrap();
    }

    pub fn write_go_time_field(&mut self, key: &str, timestamp: i64) {
        write_str(&mut self.buf, key).unwrap();
        write_ext_meta(&mut self.buf, 12, 5).unwrap();
        self.buf.extend_from_slice(&[0, 0, 0, 0]);
        self.buf.extend_from_slice(&timestamp.to_be_bytes());
    }

    pub fn write_bin_field(&mut self, key: &str, data: &[u8]) {
        write_str(&mut self.buf, key).unwrap();
        write_bin(&mut self.buf, data).unwrap();
    }

    pub fn write_array_field<F>(&mut self, key: &str, len: u32, f: F)
    where
        F: FnOnce(&mut Self),
    {
        write_str(&mut self.buf, key).unwrap();
        write_array_len(&mut self.buf, len).unwrap();
        f(self);
    }

    pub fn write_int32_array(&mut self, vals: &[i32]) {
        for &v in vals {
            write_sint(&mut self.buf, v as i64).unwrap();
        }
    }

    pub fn write_int64_array(&mut self, vals: &[i64]) {
        for &v in vals {
            if v >= i16::MIN as i64 && v <= i16::MAX as i64 {
                self.buf.push(0xd1);
                self.buf.extend_from_slice(&(v as i16).to_be_bytes());
            } else if v >= i32::MIN as i64 && v <= i32::MAX as i64 {
                self.buf.push(0xd2);
                self.buf.extend_from_slice(&(v as i32).to_be_bytes());
            } else {
                write_sint(&mut self.buf, v).unwrap();
            }
        }
    }

    pub fn write_str_array(&mut self, vals: &[String]) {
        for v in vals {
            write_str(&mut self.buf, v).unwrap();
        }
    }

    pub fn write_u8_array(&mut self, vals: &[u8]) {
        for &v in vals {
            write_uint(&mut self.buf, v as u64).unwrap();
        }
    }

    pub fn write_map_field<F>(&mut self, key: &str, len: u32, f: F)
    where
        F: FnOnce(&mut Self),
    {
        write_str(&mut self.buf, key).unwrap();
        write_map_len(&mut self.buf, len).unwrap();
        f(self);
    }

    pub fn write_bin(&mut self, data: &[u8]) {
        write_bin(&mut self.buf, data).unwrap();
    }

    pub fn finish(self) -> Vec<u8> {
        self.buf
    }
}

pub struct MsgpackReader {
    map: Vec<(Value, Value)>,
}

impl MsgpackReader {
    pub fn new(bytes: &[u8]) -> Self {
        let val = read_value(&mut Cursor::new(bytes)).unwrap();
        let map = if let Value::Map(m) = val { m } else { vec![] };
        Self { map }
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        self.map.iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .and_then(|(_, v)| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.map.iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .and_then(|(_, v)| v.as_i64())
    }

    pub fn get_u8(&self, key: &str) -> Option<u8> {
        self.map.iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .and_then(|(_, v)| v.as_u64())
            .map(|v| v as u8)
    }

    pub fn get_bytes(&self, key: &str) -> Option<Vec<u8>> {
        self.map.iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .and_then(|(_, v)| v.as_slice())
            .map(|s| s.to_vec())
    }

    pub fn get_go_time(&self, key: &str) -> Option<i64> {
        self.map.iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .and_then(|(_, v)| {
                if let Value::Ext(_, data) = v {
                    Some(i64::from_be_bytes(data[data.len()-8..].try_into().unwrap()))
                } else {
                    None
                }
            })
    }

    pub fn get_i32_array(&self, key: &str) -> Option<Vec<i32>> {
        self.map.iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .and_then(|(_, v)| {
                if let Value::Array(arr) = v {
                    Some(arr.iter().filter_map(|v| v.as_i64().map(|i| i as i32)).collect())
                } else {
                    None
                }
            })
    }

    pub fn get_i64_array(&self, key: &str) -> Option<Vec<i64>> {
        self.map.iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .and_then(|(_, v)| {
                if let Value::Array(arr) = v {
                    Some(arr.iter().filter_map(|v| v.as_i64()).collect())
                } else {
                    None
                }
            })
    }

    pub fn get_str_array(&self, key: &str) -> Option<Vec<String>> {
        self.map.iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .and_then(|(_, v)| {
                if let Value::Array(arr) = v {
                    Some(arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                } else {
                    None
                }
            })
    }

    pub fn get_u8_array(&self, key: &str) -> Option<Vec<u8>> {
        self.map.iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .and_then(|(_, v)| {
                if let Value::Array(arr) = v {
                    Some(arr.iter().filter_map(|v| v.as_u64().map(|i| i as u8)).collect())
                } else {
                    None
                }
            })
    }

    pub fn get_map(&self, key: &str) -> Option<Vec<(String, String)>> {
        self.map.iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .and_then(|(_, v)| {
                if let Value::Map(m) = v {
                    Some(m.iter()
                        .filter_map(|(k, v)| {
                            Some((k.as_str()?.to_string(), v.as_str()?.to_string()))
                        })
                        .collect())
                } else {
                    None
                }
            })
    }

    pub fn get_bytes_map(&self, key: &str) -> Option<Vec<(String, Vec<u8>)>> {
        self.map.iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .and_then(|(_, v)| {
                if let Value::Map(m) = v {
                    Some(m.iter()
                        .filter_map(|(k, v)| {
                            Some((k.as_str()?.to_string(), v.as_slice()?.to_vec()))
                        })
                        .collect())
                } else {
                    None
                }
            })
    }
}
