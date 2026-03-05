/// Golang-compatible struct encoding utilities

use super::golang_map::GoMapEncoder;

/// Wrapper for Go msgpack encoded bytes
#[derive(Debug, Clone)]
pub struct GoBytes(pub Vec<u8>);

impl From<GoBytes> for Vec<u8> {
    fn from(val: GoBytes) -> Self {
        val.0
    }
}

impl AsRef<[u8]> for GoBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Helper for encoding struct fields in Go order
pub struct GoStructBuilder {
    enc: GoMapEncoder,
}

impl GoStructBuilder {
    pub fn new(field_count: u32) -> Self {
        let mut enc = GoMapEncoder::new();
        enc.write_map_len(field_count);
        Self { enc }
    }

    pub fn field_u8(&mut self, name: &str, value: u8) {
        self.enc.write_str(name);
        self.enc.write_uint8(value);
    }

    pub fn field_i32(&mut self, name: &str, value: i32) {
        self.enc.write_str(name);
        self.enc.write_int32(value);
    }

    pub fn field_i64(&mut self, name: &str, value: i64) {
        self.enc.write_str(name);
        self.enc.write_int64_as_int16_or_larger(value);
    }

    pub fn field_i64_as_i32(&mut self, name: &str, value: i64) {
        self.enc.write_str(name);
        self.enc.write_int32(value as i32);
    }

    pub fn field_bin(&mut self, name: &str, value: &[u8]) {
        self.enc.write_str(name);
        self.enc.write_bin(value);
    }

    pub fn field_str(&mut self, name: &str, value: &str) {
        self.enc.write_str(name);
        self.enc.write_str(value);
    }

    pub fn field_array_u8(&mut self, name: &str, values: &[u8]) {
        self.enc.write_str(name);
        self.enc.write_array_len(values.len() as u32);
        for &v in values {
            self.enc.write_uint8(v);
        }
    }

    pub fn field_array_i32(&mut self, name: &str, values: &[i32]) {
        self.enc.write_str(name);
        self.enc.write_array_len(values.len() as u32);
        for &v in values {
            self.enc.write_int32(v);
        }
    }

    pub fn field_array_i64(&mut self, name: &str, values: &[i64]) {
        self.enc.write_str(name);
        self.enc.write_array_len(values.len() as u32);
        for &v in values {
            self.enc.write_int64_as_int16_or_larger(v);
        }
    }

    pub fn field_array_str(&mut self, name: &str, values: &[String]) {
        self.enc.write_str(name);
        self.enc.write_array_len(values.len() as u32);
        for v in values {
            self.enc.write_str(v);
        }
    }

    pub fn field_nil(&mut self, name: &str) {
        self.enc.write_str(name);
        self.enc.write_nil();
    }

    pub fn field_map_str_bin(&mut self, name: &str, map: &std::collections::HashMap<String, Vec<u8>>) {
        self.enc.write_str(name);
        if map.is_empty() {
            self.enc.write_nil();
        } else {
            self.enc.write_map_len(map.len() as u32);
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for k in keys {
                self.enc.write_str(k);
                self.enc.write_bin(&map[k]);
            }
        }
    }

    pub fn field_map_str_str(&mut self, name: &str, map: &std::collections::HashMap<String, String>) {
        self.enc.write_str(name);
        if map.is_empty() {
            self.enc.write_nil();
        } else {
            self.enc.write_map_len(map.len() as u32);
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for k in keys {
                self.enc.write_str(k);
                self.enc.write_str(&map[k]);
            }
        }
    }

    pub fn field_nested(&mut self, name: &str, data: &[u8]) {
        self.enc.write_str(name);
        self.enc.write_raw(data);
    }

    pub fn build(self) -> GoBytes {
        GoBytes(self.enc.into_bytes())
    }
}
