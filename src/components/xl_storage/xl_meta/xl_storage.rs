// Rust implementation of MinIO's xl-storage.go
use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::time::SystemTime;

const XL_STORAGE_FORMAT_FILE: &str = "xl.meta";
const XL_STORAGE_FORMAT_FILE_V1: &str = "xl.json";

pub struct XlStorage {
    drive_path: PathBuf,
    disk_id: String,
    format_legacy: bool,
    format_data: Vec<u8>,
}

impl XlStorage {
    pub fn new(drive_path: impl AsRef<Path>) -> io::Result<Self> {
        let path = drive_path.as_ref().to_path_buf();
        fs::create_dir_all(&path)?;
        Ok(Self {
            drive_path: path,
            disk_id: String::new(),
            format_legacy: false,
            format_data: Vec::new(),
        })
    }

    fn get_vol_dir(&self, volume: &str) -> io::Result<PathBuf> {
        if volume.is_empty() || volume == "." || volume == ".." {
            return Err(io::Error::new(io::ErrorKind::NotFound, "volume not found"));
        }
        Ok(self.drive_path.join(volume))
    }

    pub fn make_vol(&self, volume: &str) -> io::Result<()> {
        if !is_valid_volname(volume) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid argument"));
        }
        let vol_dir = self.get_vol_dir(volume)?;
        match fs::metadata(&vol_dir) {
            Ok(_) => Err(io::Error::new(io::ErrorKind::AlreadyExists, "volume exists")),
            Err(_) => {
                fs::create_dir_all(vol_dir)?;
                Ok(())
            }
        }
    }

    pub fn delete_vol(&self, volume: &str, force_delete: bool) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        if force_delete {
            if vol_dir.is_dir() {
                fs::remove_dir_all(vol_dir)?;
            } else {
                fs::remove_file(vol_dir)?;
            }
        } else {
            fs::remove_dir(vol_dir)?;
        }
        Ok(())
    }

    pub fn list_vols(&self) -> io::Result<Vec<String>> {
        let mut vols = Vec::new();
        for entry in fs::read_dir(&self.drive_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if is_valid_volname(name) {
                        vols.push(name.to_string());
                    }
                }
            }
        }
        Ok(vols)
    }

    pub fn read_all(&self, volume: &str, path: &str) -> io::Result<Vec<u8>> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        fs::read(file_path)
    }

    pub fn write_all(&self, volume: &str, path: &str, data: &[u8]) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, data)?;
        Ok(())
    }

    pub fn read_metadata_with_dmtime(&self, item_path: &Path) -> io::Result<(Vec<u8>, SystemTime)> {
        check_path_length(item_path.to_str().unwrap_or(""))?;
        let mut file = File::open(item_path)?;
        let metadata = file.metadata()?;
        if metadata.is_dir() {
            return Err(io::Error::new(io::ErrorKind::IsADirectory, "is a directory"));
        }
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok((buf, metadata.modified()?))
    }

    pub fn list_dir(&self, volume: &str, dir_path: &str, count: i32) -> io::Result<Vec<String>> {
        let vol_dir = self.get_vol_dir(volume)?;
        let full_path = vol_dir.join(dir_path);
        let mut entries = Vec::new();
        for (i, entry) in fs::read_dir(full_path)?.enumerate() {
            if count > 0 && i >= count as usize {
                break;
            }
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                let mut name = name.to_string();
                if entry.file_type()?.is_dir() {
                    name.push('/');
                }
                entries.push(name);
            }
        }
        Ok(entries)
    }

    pub fn delete_file(&self, volume: &str, path: &str, recursive: bool) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        if recursive && file_path.is_dir() {
            fs::remove_dir_all(file_path)?;
        } else {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    pub fn rename_file(&self, src_vol: &str, src_path: &str, dst_vol: &str, dst_path: &str) -> io::Result<()> {
        let src = self.get_vol_dir(src_vol)?.join(src_path);
        let dst = self.get_vol_dir(dst_vol)?.join(dst_path);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(src, dst)?;
        Ok(())
    }

    pub fn read_file(&self, volume: &str, path: &str, offset: i64, buffer: &mut [u8]) -> io::Result<i64> {
        if offset < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid argument"));
        }
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        let mut file = File::open(file_path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(io::Error::new(io::ErrorKind::Other, "not a regular file"));
        }
        file.seek(SeekFrom::Start(offset as u64))?;
        let n = file.read(buffer)?;
        Ok(n as i64)
    }

    pub fn create_file(&self, volume: &str, path: &str, _size: i64, data: &[u8]) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(file_path)?;
        file.write_all(data)?;
        Ok(())
    }

    pub fn append_file(&self, volume: &str, path: &str, buf: &[u8]) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        let mut file = OpenOptions::new().append(true).create(true).open(file_path)?;
        file.write_all(buf)?;
        Ok(())
    }

    pub fn read_version(&self, volume: &str, path: &str, version_id: &str, read_data: bool) -> io::Result<Vec<u8>> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        let xl_path = file_path.join(XL_STORAGE_FORMAT_FILE);
        let buf = fs::read(&xl_path).or_else(|_| {
            if self.format_legacy {
                fs::read(file_path.join(XL_STORAGE_FORMAT_FILE_V1))
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "file not found"))
            }
        })?;
        Ok(buf)
    }

    pub fn delete_version(&self, volume: &str, path: &str) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        let xl_path = file_path.join(XL_STORAGE_FORMAT_FILE);
        fs::remove_file(&xl_path).or_else(|_| {
            if self.format_legacy {
                fs::remove_file(file_path.join(XL_STORAGE_FORMAT_FILE_V1))
            } else {
                Ok(())
            }
        })?;
        let _ = fs::remove_dir(&file_path);
        Ok(())
    }

    pub fn write_metadata_fresh(&self, volume: &str, path: &str, data: &[u8]) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path).join(XL_STORAGE_FORMAT_FILE);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, data)?;
        Ok(())
    }

    pub fn delete_bulk(&self, volume: &str, paths: &[&str]) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        for path in paths {
            let file_path = vol_dir.join(path);
            check_path_length(file_path.to_str().unwrap_or(""))?;
            let _ = fs::remove_file(&file_path).or_else(|_| fs::remove_dir_all(&file_path));
        }
        Ok(())
    }

    pub fn stat_info_file(&self, volume: &str, path: &str) -> io::Result<(u64, SystemTime)> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        let metadata = fs::metadata(file_path)?;
        Ok((metadata.len(), metadata.modified()?))
    }

    pub fn rename_data(&self, src_vol: &str, src_path: &str, dst_vol: &str, dst_path: &str) -> io::Result<()> {
        let src = self.get_vol_dir(src_vol)?.join(src_path);
        let dst = self.get_vol_dir(dst_vol)?.join(dst_path);
        check_path_length(src.to_str().unwrap_or(""))?;
        check_path_length(dst.to_str().unwrap_or(""))?;
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(src, dst)?;
        Ok(())
    }

    pub fn check_parts(&self, volume: &str, path: &str, data_dir: &str, parts: &[(i32, i64)]) -> io::Result<Vec<bool>> {
        let vol_dir = self.get_vol_dir(volume)?;
        let mut results = Vec::with_capacity(parts.len());
        for (part_num, expected_size) in parts {
            let part_path = vol_dir.join(path).join(data_dir).join(format!("part.{}", part_num));
            match fs::metadata(&part_path) {
                Ok(meta) if meta.is_file() && meta.len() >= *expected_size as u64 => results.push(true),
                _ => results.push(false),
            }
        }
        Ok(results)
    }

    pub fn read_file_stream(&self, volume: &str, path: &str, offset: i64, length: i64) -> io::Result<File> {
        if offset < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid argument"));
        }
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        let mut file = File::open(file_path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(io::Error::new(io::ErrorKind::Other, "not a regular file"));
        }
        if length >= 0 && metadata.len() < (offset + length) as u64 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "file corrupt"));
        }
        if offset > 0 {
            file.seek(SeekFrom::Start(offset as u64))?;
        }
        Ok(file)
    }

    pub fn read_xl(&self, volume: &str, path: &str, read_data: bool) -> io::Result<Vec<u8>> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        let xl_path = file_path.join(XL_STORAGE_FORMAT_FILE);
        fs::read(&xl_path).or_else(|_| {
            if self.format_legacy {
                fs::read(file_path.join(XL_STORAGE_FORMAT_FILE_V1))
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "file not found"))
            }
        })
    }

    pub fn clean_abandoned_data(&self, volume: &str, path: &str) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        if file_path.exists() {
            fs::remove_dir_all(file_path)?;
        }
        Ok(())
    }

    pub fn verify_file(&self, volume: &str, path: &str, data_dir: &str, parts: &[(i32, i64)]) -> io::Result<bool> {
        let vol_dir = self.get_vol_dir(volume)?;
        for (part_num, expected_size) in parts {
            let part_path = vol_dir.join(path).join(data_dir).join(format!("part.{}", part_num));
            match fs::metadata(&part_path) {
                Ok(meta) if meta.is_file() && meta.len() >= *expected_size as u64 => continue,
                _ => return Ok(false),
            }
        }
        Ok(true)
    }

    pub fn rename_part(&self, src_vol: &str, src_path: &str, dst_vol: &str, dst_path: &str, meta: &[u8]) -> io::Result<()> {
        let src = self.get_vol_dir(src_vol)?.join(src_path);
        let dst = self.get_vol_dir(dst_vol)?.join(dst_path);
        check_path_length(src.to_str().unwrap_or(""))?;
        check_path_length(dst.to_str().unwrap_or(""))?;
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&src, &dst)?;
        if !meta.is_empty() {
            let meta_path = dst.join(XL_STORAGE_FORMAT_FILE);
            fs::write(meta_path, meta)?;
        }
        Ok(())
    }

    pub fn read_multiple(&self, volume: &str, paths: &[&str]) -> io::Result<Vec<Option<Vec<u8>>>> {
        let vol_dir = self.get_vol_dir(volume)?;
        let mut results = Vec::with_capacity(paths.len());
        for path in paths {
            let file_path = vol_dir.join(path);
            match fs::read(file_path) {
                Ok(data) => results.push(Some(data)),
                Err(_) => results.push(None),
            }
        }
        Ok(results)
    }

    pub fn stat_vol(&self, volume: &str) -> io::Result<SystemTime> {
        let vol_dir = self.get_vol_dir(volume)?;
        let metadata = fs::metadata(vol_dir)?;
        metadata.modified()
    }

    pub fn walk_dir(&self, volume: &str, prefix: &str, marker: &str, recursive: bool) -> io::Result<Vec<String>> {
        let vol_dir = self.get_vol_dir(volume)?;
        let prefix_path = vol_dir.join(prefix);
        let mut entries = Vec::new();
        if let Ok(read_dir) = fs::read_dir(prefix_path) {
            for entry in read_dir.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if !marker.is_empty() && name <= marker {
                        continue;
                    }
                    entries.push(name.to_string());
                }
            }
        }
        entries.sort();
        Ok(entries)
    }

    pub fn get_disk_id(&self) -> &str {
        &self.disk_id
    }

    pub fn is_online(&self) -> bool {
        self.drive_path.exists()
    }

    pub fn delete_file_internal(&self, base_path: &Path, delete_path: &Path, recursive: bool) -> io::Result<()> {
        if !delete_path.starts_with(base_path) || delete_path == base_path {
            return Ok(());
        }
        if recursive && delete_path.is_dir() {
            fs::remove_dir_all(delete_path)?;
        } else {
            fs::remove_file(delete_path).or_else(|_| fs::remove_dir(delete_path))?;
        }
        if let Some(parent) = delete_path.parent() {
            let _ = self.delete_file_internal(base_path, parent, false);
        }
        Ok(())
    }

    pub fn make_vol_bulk(&self, volumes: &[&str]) -> io::Result<()> {
        for vol in volumes {
            let _ = self.make_vol(vol);
        }
        Ok(())
    }

    pub fn delete_versions(&self, volume: &str, path: &str, version_ids: &[&str]) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        let xl_path = file_path.join(XL_STORAGE_FORMAT_FILE);
        let buf = fs::read(&xl_path)?;
        // TODO: Parse xl.meta, remove versions, write back
        Ok(())
    }

    pub fn update_metadata(&self, volume: &str, path: &str, metadata: &[u8]) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path).join(XL_STORAGE_FORMAT_FILE);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        fs::write(file_path, metadata)?;
        Ok(())
    }

    pub fn read_parts(&self, volume: &str, part_paths: &[&str]) -> io::Result<Vec<Option<Vec<u8>>>> {
        let vol_dir = self.get_vol_dir(volume)?;
        let mut results = Vec::with_capacity(part_paths.len());
        for path in part_paths {
            let file_path = vol_dir.join(path);
            match fs::read(file_path) {
                Ok(data) => results.push(Some(data)),
                Err(_) => results.push(None),
            }
        }
        Ok(results)
    }

    pub fn write_all_meta(&self, volume: &str, path: &str, data: &[u8], sync: bool) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        check_path_length(file_path.to_str().unwrap_or(""))?;
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, data)?;
        Ok(())
    }

    pub fn read_all_data_with_dmtime(&self, volume: &str, file_path: &Path) -> io::Result<(Vec<u8>, SystemTime)> {
        let mut file = File::open(file_path)?;
        let metadata = file.metadata()?;
        if metadata.is_dir() {
            return Err(io::Error::new(io::ErrorKind::IsADirectory, "is directory"));
        }
        let mut buf = Vec::with_capacity(metadata.len() as usize);
        file.read_to_end(&mut buf)?;
        Ok((buf, metadata.modified()?))
    }

    pub fn open_file_direct(&self, path: &Path, write: bool) -> io::Result<File> {
        if write {
            OpenOptions::new().write(true).create(true).truncate(true).open(path)
        } else {
            File::open(path)
        }
    }

    pub fn write_all_direct(&self, file_path: &Path, data: &[u8]) -> io::Result<()> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(file_path)?;
        file.write_all(data)?;
        file.sync_all()?;
        Ok(())
    }

    pub fn read_raw(&self, volume: &str, path: &str, read_data: bool) -> io::Result<Vec<u8>> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        let xl_path = file_path.join(XL_STORAGE_FORMAT_FILE);
        fs::read(&xl_path).or_else(|_| {
            if self.format_legacy {
                fs::read(file_path.join(XL_STORAGE_FORMAT_FILE_V1))
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "file not found"))
            }
        })
    }

    pub fn rename_legacy_metadata(&self, volume_dir: &Path, path: &str) -> io::Result<()> {
        if !self.format_legacy {
            return Err(io::Error::new(io::ErrorKind::NotFound, "not legacy"));
        }
        let file_path = volume_dir.join(path);
        let src = file_path.join(XL_STORAGE_FORMAT_FILE_V1);
        let dst = file_path.join(XL_STORAGE_FORMAT_FILE);
        fs::rename(src, dst)?;
        Ok(())
    }

    pub fn disk_info(&self) -> io::Result<(u64, u64)> {
        // Returns (total, free) space
        // TODO: Implement platform-specific disk info
        Ok((0, 0))
    }

    pub fn check_format_json(&self) -> io::Result<bool> {
        let format_file = self.drive_path.join(".minio.sys").join("format.json");
        Ok(format_file.exists())
    }

    pub fn healing(&self) -> io::Result<bool> {
        let healing_file = self.drive_path.join(".minio.sys").join("buckets").join(".healing.bin");
        Ok(healing_file.exists())
    }

    pub fn write_all_internal(&self, file_path: &Path, data: &[u8], sync: bool) -> io::Result<()> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(file_path)?;
        file.write_all(data)?;
        if sync {
            file.sync_all()?;
        }
        Ok(())
    }

    pub fn open_file_sync(&self, file_path: &Path, write: bool) -> io::Result<File> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if write {
            OpenOptions::new().write(true).create(true).open(file_path)
        } else {
            File::open(file_path)
        }
    }

    pub fn bitrot_verify(&self, _path: &Path, _size: i64, _algo: u8, _sum: &[u8]) -> io::Result<bool> {
        // TODO: Implement bitrot verification with HighwayHash
        Ok(true)
    }

    pub fn read_all_with_timeout(&self, volume: &str, path: &str) -> io::Result<Vec<u8>> {
        self.read_all(volume, path)
    }

    pub fn write_with_verify(&self, volume: &str, path: &str, data: &[u8]) -> io::Result<()> {
        self.write_all(volume, path, data)
    }

    pub fn stat_file(&self, volume: &str, path: &str) -> io::Result<(u64, SystemTime, bool)> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        let metadata = fs::metadata(file_path)?;
        Ok((metadata.len(), metadata.modified()?, metadata.is_dir()))
    }

    pub fn list_dir_with_opts(&self, volume: &str, dir_path: &str, count: i32, leaf: bool) -> io::Result<Vec<String>> {
        self.list_dir(volume, dir_path, count)
    }

    pub fn access(&self, path: &Path) -> io::Result<()> {
        if path.exists() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "not found"))
        }
    }

    pub fn copy_file(&self, src_vol: &str, src_path: &str, dst_vol: &str, dst_path: &str) -> io::Result<()> {
        let src = self.get_vol_dir(src_vol)?.join(src_path);
        let dst = self.get_vol_dir(dst_vol)?.join(dst_path);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        Ok(())
    }

    pub fn truncate_file(&self, volume: &str, path: &str, size: u64) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        let file = OpenOptions::new().write(true).open(file_path)?;
        file.set_len(size)?;
        Ok(())
    }

    pub fn read_at(&self, volume: &str, path: &str, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        let mut file = File::open(file_path)?;
        file.seek(SeekFrom::Start(offset))?;
        file.read(buf)
    }

    pub fn write_at(&self, volume: &str, path: &str, offset: u64, data: &[u8]) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        let mut file = OpenOptions::new().write(true).create(true).open(file_path)?;
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(data)?;
        Ok(())
    }

    pub fn sync_file(&self, volume: &str, path: &str) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        let file = OpenOptions::new().write(true).open(file_path)?;
        file.sync_all()?;
        Ok(())
    }

    pub fn link_file(&self, src_vol: &str, src_path: &str, dst_vol: &str, dst_path: &str) -> io::Result<()> {
        let src = self.get_vol_dir(src_vol)?.join(src_path);
        let dst = self.get_vol_dir(dst_vol)?.join(dst_path);
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(src, dst)?;
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(src, dst)?;
        }
        Ok(())
    }

    pub fn exists(&self, volume: &str, path: &str) -> bool {
        if let Ok(vol_dir) = self.get_vol_dir(volume) {
            vol_dir.join(path).exists()
        } else {
            false
        }
    }

    pub fn is_dir(&self, volume: &str, path: &str) -> bool {
        if let Ok(vol_dir) = self.get_vol_dir(volume) {
            vol_dir.join(path).is_dir()
        } else {
            false
        }
    }

    pub fn is_file(&self, volume: &str, path: &str) -> bool {
        if let Ok(vol_dir) = self.get_vol_dir(volume) {
            vol_dir.join(path).is_file()
        } else {
            false
        }
    }

    pub fn get_file_size(&self, volume: &str, path: &str) -> io::Result<u64> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        let metadata = fs::metadata(file_path)?;
        Ok(metadata.len())
    }

    pub fn get_modified_time(&self, volume: &str, path: &str) -> io::Result<SystemTime> {
        let vol_dir = self.get_vol_dir(volume)?;
        let file_path = vol_dir.join(path);
        let metadata = fs::metadata(file_path)?;
        metadata.modified()
    }

    pub fn create_dir(&self, volume: &str, path: &str) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let dir_path = vol_dir.join(path);
        fs::create_dir_all(dir_path)?;
        Ok(())
    }

    pub fn remove_dir(&self, volume: &str, path: &str, recursive: bool) -> io::Result<()> {
        let vol_dir = self.get_vol_dir(volume)?;
        let dir_path = vol_dir.join(path);
        if recursive {
            fs::remove_dir_all(dir_path)?;
        } else {
            fs::remove_dir(dir_path)?;
        }
        Ok(())
    }
}

pub fn is_valid_volname(volname: &str) -> bool {
    if volname.len() < 3 {
        return false;
    }
    #[cfg(target_os = "windows")]
    {
        !volname.contains(&['\\', ':', '*', '?', '"', '<', '>', '|'][..])
    }
    #[cfg(not(target_os = "windows"))]
    {
        true
    }
}

pub fn check_path_length(path: &str) -> io::Result<()> {
    #[cfg(target_os = "macos")]
    if path.len() > 1016 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "path too long"));
    }
    #[cfg(target_os = "windows")]
    if path.len() > 1024 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "path too long"));
    }
    if path == "." || path == ".." || path == "/" {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "access denied"));
    }
    for segment in path.split('/') {
        if segment.len() > 255 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "segment too long"));
        }
    }
    Ok(())
}
