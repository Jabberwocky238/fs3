use std::collections::{HashMap, HashSet};
use std::io::{self, Cursor, Read};
use std::path::PathBuf;
use std::sync::RwLock;

use chrono::Utc;

use super::{ListResult, MountError, ObjectInfo};

#[derive(Debug, Default)]
pub struct MemoryMountManager {
    buckets: RwLock<HashMap<String, HashMap<String, Vec<u8>>>>,
}

impl MemoryMountManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_data(&self, bucket: &str, key: &str) -> Result<Vec<u8>, MountError> {
        let guard = self.buckets.read().expect("poisoned lock");
        let b = guard.get(bucket).ok_or(MountError::NoSuchBucket)?;
        b.get(key).cloned().ok_or(MountError::NoSuchKey)
    }
}

impl crate::mount::MountManager for MemoryMountManager {
    fn buckets(&self) -> Vec<String> {
        let guard = self.buckets.read().expect("poisoned lock");
        let mut out: Vec<String> = guard.keys().cloned().collect();
        out.sort();
        out
    }

    fn has_bucket(&self, bucket: &str) -> bool {
        self.buckets
            .read()
            .expect("poisoned lock")
            .contains_key(bucket)
    }

    fn ensure_bucket(&self, bucket: &str) -> Result<(), MountError> {
        let mut guard = self.buckets.write().expect("poisoned lock");
        guard.entry(bucket.to_string()).or_default();
        Ok(())
    }

    fn open(&self, bucket: &str, key: &str) -> Result<(Box<dyn Read + Send>, ObjectInfo), MountError> {
        let info = self.stat(bucket, key)?;
        let data = self.get_data(bucket, &info.key)?;
        Ok((Box::new(Cursor::new(data)), info))
    }

    fn stat(&self, bucket: &str, key: &str) -> Result<ObjectInfo, MountError> {
        let key = normalize_key(key)?;
        let guard = self.buckets.read().expect("poisoned lock");
        let b = guard.get(bucket).ok_or(MountError::NoSuchBucket)?;
        let data = b.get(&key).ok_or(MountError::NoSuchKey)?;
        Ok(ObjectInfo {
            key: key.clone(),
            size: data.len() as i64,
            last_modified: Utc::now(),
            physical_path: PathBuf::from(format!("/__mem__/{bucket}/{key}")),
        })
    }

    fn put(&self, bucket: &str, key: &str, data: &[u8]) -> Result<ObjectInfo, MountError> {
        self.put_reader(bucket, key, &mut Cursor::new(data))
    }

    fn put_reader(
        &self,
        bucket: &str,
        key: &str,
        r: &mut dyn Read,
    ) -> Result<ObjectInfo, MountError> {
        let key = normalize_key(key)?;
        let mut body = Vec::new();
        io::copy(r, &mut body)?;

        let mut guard = self.buckets.write().expect("poisoned lock");
        let b = guard.get_mut(bucket).ok_or(MountError::NoSuchBucket)?;
        let size = body.len() as i64;
        b.insert(key.clone(), body);
        Ok(ObjectInfo {
            key: key.clone(),
            size,
            last_modified: Utc::now(),
            physical_path: PathBuf::from(format!("/__mem__/{bucket}/{key}")),
        })
    }

    fn delete(&self, bucket: &str, key: &str) -> Result<(), MountError> {
        let key = normalize_key(key)?;
        let mut guard = self.buckets.write().expect("poisoned lock");
        let b = guard.get_mut(bucket).ok_or(MountError::NoSuchBucket)?;
        if b.remove(&key).is_some() {
            Ok(())
        } else {
            Err(MountError::NoSuchKey)
        }
    }

    fn list(
        &self,
        bucket: &str,
        prefix: &str,
        delimiter: &str,
        token: &str,
        max_keys: usize,
    ) -> Result<ListResult, MountError> {
        let max_keys = if max_keys == 0 { 1000 } else { max_keys };
        let guard = self.buckets.read().expect("poisoned lock");
        let b = guard.get(bucket).ok_or(MountError::NoSuchBucket)?;

        let mut keys: Vec<ObjectInfo> = b
            .iter()
            .filter_map(|(k, v)| {
                if k.starts_with(prefix) {
                    Some(ObjectInfo {
                        key: k.clone(),
                        size: v.len() as i64,
                        last_modified: Utc::now(),
                        physical_path: PathBuf::from(format!("/__mem__/{bucket}/{k}")),
                    })
                } else {
                    None
                }
            })
            .collect();
        keys.sort_by(|a, b| a.key.cmp(&b.key));

        let mut filtered = Vec::new();
        let mut cps = Vec::new();
        let mut cp_set = HashSet::new();

        for obj in keys {
            if !token.is_empty() && obj.key.as_str() <= token {
                continue;
            }
            if !delimiter.is_empty() {
                let rest = obj.key.strip_prefix(prefix).unwrap_or(&obj.key);
                if let Some(i) = rest.find(delimiter) {
                    let cp = format!("{}{}", prefix, &rest[..i + 1]);
                    if cp_set.insert(cp.clone()) {
                        cps.push(cp);
                    }
                    continue;
                }
            }
            filtered.push(obj);
        }
        cps.sort();

        let merged = filtered.len() + cps.len();
        if merged <= max_keys {
            return Ok(ListResult {
                keys: filtered,
                common_prefixes: cps,
                next_token: String::new(),
                truncated: false,
            });
        }

        let mut out = ListResult {
            truncated: true,
            ..Default::default()
        };
        let mut i = 0usize;
        let mut j = 0usize;
        let mut count = 0usize;
        let mut last = String::new();
        while count < max_keys && (i < filtered.len() || j < cps.len()) {
            if j >= cps.len() || (i < filtered.len() && filtered[i].key <= cps[j]) {
                last = filtered[i].key.clone();
                out.keys.push(filtered[i].clone());
                i += 1;
            } else {
                last = cps[j].clone();
                out.common_prefixes.push(cps[j].clone());
                j += 1;
            }
            count += 1;
        }
        out.next_token = last;
        Ok(out)
    }
}

fn normalize_key(key: &str) -> Result<String, MountError> {
    let k = key.trim().replace('\\', "/");
    if k.is_empty() {
        return Err(MountError::BadKey);
    }
    let mut parts = Vec::new();
    for seg in k.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            return Err(MountError::BadKey);
        }
        parts.push(seg);
    }
    if parts.is_empty() {
        return Err(MountError::BadKey);
    }
    Ok(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::MemoryMountManager;
    use crate::mount::MountManager as _;
    use std::io::Read;

    #[test]
    fn memory_mount_basic_flow() {
        let mounts = MemoryMountManager::new();
        mounts.ensure_bucket("docs").expect("ensure bucket");
        mounts.put("docs", "a/b.txt", b"hello").expect("put");
        let mut opened = mounts.open("docs", "a/b.txt").expect("open").0;
        let mut body = String::new();
        opened.read_to_string(&mut body).expect("read");
        assert_eq!(body, "hello");
        let listed = mounts.list("docs", "a/", "", "", 100).expect("list");
        assert_eq!(listed.keys.len(), 1);
        mounts.delete("docs", "a/b.txt").expect("delete");
    }
}
