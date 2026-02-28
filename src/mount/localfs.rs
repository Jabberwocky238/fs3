use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use tracing::info;

use super::{ListResult, MountError, ObjectInfo};

#[derive(Debug, Clone)]
pub struct LocalFsMountManager {
    root: PathBuf,
}

impl LocalFsMountManager {
    pub fn new(path: &str) -> Result<Self, MountError> {
        let root = absolute_path(Path::new(path))?;
        fs::create_dir_all(&root)?;
        info!(root = %root.display(), "localfs mount initialized");
        Ok(Self { root })
    }

    fn bucket_dir(&self, bucket: &str) -> PathBuf {
        self.root.join(bucket)
    }
}

impl crate::mount::MountManager for LocalFsMountManager {
    fn buckets(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                if entry.path().is_dir()
                    && let Some(name) = entry.file_name().to_str()
                {
                    out.push(name.to_string());
                }
            }
        }
        out.sort();
        out
    }

    fn has_bucket(&self, bucket: &str) -> bool {
        self.bucket_dir(bucket).is_dir()
    }

    fn ensure_bucket(&self, bucket: &str) -> Result<(), MountError> {
        fs::create_dir_all(self.bucket_dir(bucket))?;
        Ok(())
    }

    fn open(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(Box<dyn Read + Send>, ObjectInfo), MountError> {
        let obj = self.stat(bucket, key)?;
        let f = File::open(&obj.physical_path)?;
        Ok((Box::new(f), obj))
    }

    fn stat(&self, bucket: &str, key: &str) -> Result<ObjectInfo, MountError> {
        let key = normalize_key(key)?;
        let dir = self.bucket_dir(bucket);
        if !dir.is_dir() {
            return Err(MountError::NoSuchBucket);
        }
        let p = dir.join(key_to_rel(&key));
        let md = fs::metadata(&p).map_err(|_| MountError::NoSuchKey)?;
        if md.is_dir() {
            return Err(MountError::NoSuchKey);
        }
        Ok(ObjectInfo {
            key,
            size: md.len() as i64,
            last_modified: DateTime::<Utc>::from(
                md.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            ),
            physical_path: p,
        })
    }

    fn put(&self, bucket: &str, key: &str, data: &[u8]) -> Result<ObjectInfo, MountError> {
        self.put_reader(bucket, key, &mut io::Cursor::new(data))
    }

    fn put_reader(
        &self,
        bucket: &str,
        key: &str,
        r: &mut dyn Read,
    ) -> Result<ObjectInfo, MountError> {
        let key = normalize_key(key)?;
        let dir = self.bucket_dir(bucket);
        if !dir.is_dir() {
            return Err(MountError::NoSuchBucket);
        }
        let p = dir.join(key_to_rel(&key));
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = File::create(&p)?;
        io::copy(r, &mut f)?;
        self.stat(bucket, &key)
    }

    fn delete(&self, bucket: &str, key: &str) -> Result<(), MountError> {
        let key = normalize_key(key)?;
        let dir = self.bucket_dir(bucket);
        if !dir.is_dir() {
            return Err(MountError::NoSuchBucket);
        }
        let p = dir.join(key_to_rel(&key));
        match fs::remove_file(p) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Err(MountError::NoSuchKey),
            Err(e) => Err(MountError::Io(e)),
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
        let dir = self.bucket_dir(bucket);
        if !dir.is_dir() {
            return Err(MountError::NoSuchBucket);
        }
        let max_keys = if max_keys == 0 { 1000 } else { max_keys };
        let all = walk_objects(&dir)?;

        let mut keys = Vec::new();
        let mut cp_set = HashSet::new();
        let mut cps = Vec::new();
        let mut started = token.is_empty();

        for obj in all {
            if !started && obj.key.as_str() <= token {
                continue;
            }
            if !obj.key.starts_with(prefix) {
                continue;
            }
            started = true;

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
            keys.push(obj);
        }

        cps.sort();
        let merged = keys.len() + cps.len();
        if merged <= max_keys {
            return Ok(ListResult {
                keys,
                common_prefixes: cps,
                next_token: String::new(),
                truncated: false,
            });
        }

        let mut out = ListResult {
            truncated: true,
            ..Default::default()
        };
        let (mut i, mut j, mut count) = (0usize, 0usize, 0usize);
        let mut last = String::new();
        while count < max_keys && (i < keys.len() || j < cps.len()) {
            if j >= cps.len() || (i < keys.len() && keys[i].key <= cps[j]) {
                last.clone_from(&keys[i].key);
                out.keys.push(keys[i].clone());
                i += 1;
            } else {
                last.clone_from(&cps[j]);
                out.common_prefixes.push(cps[j].clone());
                j += 1;
            }
            count += 1;
        }
        out.next_token = last;
        Ok(out)
    }
}

fn walk_objects(root: &Path) -> Result<Vec<ObjectInfo>, MountError> {
    if !root.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let p = entry.path();
            let md = entry.metadata()?;
            if md.is_dir() {
                stack.push(p);
                continue;
            }
            let rel = p
                .strip_prefix(root)
                .map_err(|e| MountError::Config(e.to_string()))?;
            let key = rel.to_string_lossy().replace('\\', "/");
            out.push(ObjectInfo {
                key,
                size: md.len() as i64,
                last_modified: DateTime::<Utc>::from(
                    md.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                ),
                physical_path: p,
            });
        }
    }
    out.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(out)
}

fn absolute_path(p: &Path) -> Result<PathBuf, MountError> {
    if p.is_absolute() {
        Ok(p.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(p))
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

fn key_to_rel(key: &str) -> PathBuf {
    key.split('/').fold(PathBuf::new(), |mut acc, s| {
        acc.push(s);
        acc
    })
}
