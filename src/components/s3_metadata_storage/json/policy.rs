use async_trait::async_trait;

use crate::types::errors::S3MetadataStorageError;
use crate::types::traits::s3_metadata_storage::S3MetadataStoragePolicy;

use super::JsonMetadataStorage;

type Result<T> = std::result::Result<T, S3MetadataStorageError>;

fn upsert(vec: &mut Vec<(String, String)>, key: &str, val: &str) {
    if let Some(entry) = vec.iter_mut().find(|(k, _)| k == key) {
        entry.1 = val.to_owned();
    } else {
        vec.push((key.to_owned(), val.to_owned()));
    }
}

fn remove(vec: &mut Vec<(String, String)>, key: &str) {
    vec.retain(|(k, _)| k != key);
}

fn find(vec: &[(String, String)], key: &str) -> Option<String> {
    vec.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

#[async_trait]
impl S3MetadataStoragePolicy for JsonMetadataStorage {
    async fn store_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<()> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        upsert(&mut snap.bucket_policies, bucket, policy_json);
        self.save_sync(&snap)
    }

    async fn load_bucket_policy(&self, bucket: &str) -> Result<Option<String>> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(find(&snap.bucket_policies, bucket))
    }

    async fn delete_bucket_policy(&self, bucket: &str) -> Result<()> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        remove(&mut snap.bucket_policies, bucket);
        self.save_sync(&snap)
    }

    async fn store_iam_policy(&self, policy_name: &str, policy_json: &str) -> Result<()> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        upsert(&mut snap.iam_policies, policy_name, policy_json);
        self.save_sync(&snap)
    }

    async fn load_iam_policy(&self, policy_name: &str) -> Result<Option<String>> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(find(&snap.iam_policies, policy_name))
    }

    async fn delete_iam_policy(&self, policy_name: &str) -> Result<()> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        remove(&mut snap.iam_policies, policy_name);
        self.save_sync(&snap)
    }

    async fn list_iam_policies(&self) -> Result<Vec<String>> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(snap.iam_policies.iter().map(|(k, _)| k.clone()).collect())
    }

    async fn store_user_policy_mapping(&self, identity: &str, policy_names: &str) -> Result<()> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        upsert(&mut snap.user_policy_mappings, identity, policy_names);
        self.save_sync(&snap)
    }

    async fn load_user_policy_mapping(&self, identity: &str) -> Result<Option<String>> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(find(&snap.user_policy_mappings, identity))
    }

    async fn delete_user_policy_mapping(&self, identity: &str) -> Result<()> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        remove(&mut snap.user_policy_mappings, identity);
        self.save_sync(&snap)
    }

    async fn store_group_policy_mapping(&self, group: &str, policy_names: &str) -> Result<()> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        upsert(&mut snap.group_policy_mappings, group, policy_names);
        self.save_sync(&snap)
    }

    async fn load_group_policy_mapping(&self, group: &str) -> Result<Option<String>> {
        let _lock = self.lock.lock().await;
        let snap = self.load_sync()?;
        Ok(find(&snap.group_policy_mappings, group))
    }

    async fn delete_group_policy_mapping(&self, group: &str) -> Result<()> {
        let _lock = self.lock.lock().await;
        let mut snap = self.load_sync()?;
        remove(&mut snap.group_policy_mappings, group);
        self.save_sync(&snap)
    }
}
