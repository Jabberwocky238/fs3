#![cfg(feature = "storage-k8sconfigmap")]


use crate::policy::PolicyGroup;
use crate::storage::BucketMetaStore;
use crate::storage::PolicyStore;
#[cfg(feature = "multi-user")]
use crate::storage::UserStore;
#[cfg(feature = "multi-user")]
use crate::storage::types::UserRecord;
use crate::storage::types::{BucketMetadata, StorageError, StorageSnapshot};

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

#[cfg_attr(target_os = "windows", link(name = "libfs3k8sconfigmap", kind = "raw-dylib"))]
unsafe extern "C" {
    fn InitConfigMapClient(namespace: *const c_char) -> *mut c_char;
    fn EnsureConfigMapExists(name: *const c_char, data: *const c_char) -> *mut c_char;
    fn ReadConfigMap(name: *const c_char, out_data: *mut *mut c_char) -> *mut c_char;
    fn WriteConfigMap(name: *const c_char, data: *const c_char) -> *mut c_char;
    fn FreeCString(ptr: *mut c_char);
}

/// Convert a C error string pointer into a `Result`. Frees the C string on error.
unsafe fn check_err(ptr: *mut c_char) -> Result<(), StorageError> {
    if ptr.is_null() {
        return Ok(());
    }
    let msg = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned();
    unsafe { FreeCString(ptr) };
    Err(StorageError::Db(msg))
}

pub struct ConfigMapStore {
    name: CString,
}

impl ConfigMapStore {
    pub async fn new(name: String, seed: StorageSnapshot) -> Result<Self, StorageError> {
        let (namespace, cm_name) = parse_name(&name)?;

        let c_namespace = CString::new(namespace)
            .map_err(|e| StorageError::Io(e.to_string()))?;
        let c_name = CString::new(cm_name)
            .map_err(|e| StorageError::Io(e.to_string()))?;

        unsafe { check_err(InitConfigMapClient(c_namespace.as_ptr()))? };

        let seed_json = serde_json::to_string(&seed)
            .map_err(|e| StorageError::Serde(e.to_string()))?;
        let c_seed = CString::new(seed_json)
            .map_err(|e| StorageError::Io(e.to_string()))?;

        unsafe { check_err(EnsureConfigMapExists(c_name.as_ptr(), c_seed.as_ptr()))? };

        Ok(Self { name: c_name })
    }

    async fn read_snapshot(&self) -> Result<StorageSnapshot, StorageError> {
        let mut out: *mut c_char = ptr::null_mut();
        unsafe {
            check_err(ReadConfigMap(self.name.as_ptr(), &mut out))?;
            let json = CStr::from_ptr(out).to_string_lossy().into_owned();
            FreeCString(out);
            serde_json::from_str(&json).map_err(|e| StorageError::Serde(e.to_string()))
        }
    }

    async fn write_snapshot(&self, snap: &StorageSnapshot) -> Result<(), StorageError> {
        let body = serde_json::to_string(snap)
            .map_err(|e| StorageError::Serde(e.to_string()))?;
        let c_body = CString::new(body)
            .map_err(|e| StorageError::Io(e.to_string()))?;
        unsafe { check_err(WriteConfigMap(self.name.as_ptr(), c_body.as_ptr())) }
    }
}

#[cfg(feature = "multi-user")]
#[async_trait::async_trait]
impl UserStore for ConfigMapStore {
    async fn list_users(&self) -> Result<Vec<UserRecord>, StorageError> {
        Ok(self.read_snapshot().await?.users)
    }

    async fn save_users(&self, users: Vec<UserRecord>) -> Result<(), StorageError> {
        let mut snap = self.read_snapshot().await?;
        snap.users = users;
        self.write_snapshot(&snap).await
    }
}


#[async_trait::async_trait]
impl PolicyStore for ConfigMapStore {
    async fn list_policy_groups(&self) -> Result<Vec<PolicyGroup>, StorageError> {
        Ok(self.read_snapshot().await?.policies)
    }

    async fn save_policy_groups(&self, groups: Vec<PolicyGroup>) -> Result<(), StorageError> {
        let mut snap = self.read_snapshot().await?;
        snap.policies = groups;
        self.write_snapshot(&snap).await
    }
}

#[async_trait::async_trait]
impl BucketMetaStore for ConfigMapStore {
    async fn list_bucket_metadata(&self) -> Result<Vec<BucketMetadata>, StorageError> {
        Ok(self.read_snapshot().await?.bucket_metadata)
    }

    async fn upsert_bucket_metadata(&self, meta: BucketMetadata) -> Result<(), StorageError> {
        let mut snap = self.read_snapshot().await?;
        if let Some(item) = snap
            .bucket_metadata
            .iter_mut()
            .find(|x| x.bucket == meta.bucket)
        {
            *item = meta;
        } else {
            snap.bucket_metadata.push(meta);
        }
        self.write_snapshot(&snap).await
    }
}

fn parse_name(input: &str) -> Result<(String, String), StorageError> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err(StorageError::Io("configmap_name is empty".to_string()));
    }
    if let Some((ns, name)) = raw.split_once('/') {
        if ns.trim().is_empty() || name.trim().is_empty() {
            return Err(StorageError::Io(
                "configmap_name must be 'name' or 'namespace/name'".to_string(),
            ));
        }
        return Ok((ns.trim().to_string(), name.trim().to_string()));
    }
    Ok(("default".to_string(), raw.to_string()))
}
