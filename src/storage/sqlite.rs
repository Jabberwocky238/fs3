#![cfg(feature = "storage-sqlite")]

use std::collections::HashSet;

#[cfg(feature = "policy")]
use crate::config::PolicyGroup;
use crate::storage::BucketMetaStore;
#[cfg(feature = "policy")]
use crate::storage::PolicyStore;
#[cfg(feature = "multi-user")]
use crate::storage::UserStore;
#[cfg(feature = "multi-user")]
use crate::storage::types::UserRecord;
use crate::storage::types::{BucketMetadata, StorageError, StorageSnapshot};

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn new(dsn: String, seed: StorageSnapshot) -> Result<Self, StorageError> {
        if dsn.trim().is_empty() {
            return Err(StorageError::Io("sqlite dsn is empty".to_string()));
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&dsn)
            .await
            .map_err(|e| StorageError::Db(e.to_string()))?;

        let s = Self { pool };
        s.ensure_tables().await?;
        s.seed_if_empty(seed).await?;
        Ok(s)
    }

    async fn ensure_tables(&self) -> Result<(), StorageError> {
        sqlx::query("CREATE TABLE IF NOT EXISTS users (user_id TEXT PRIMARY KEY, enabled INTEGER NOT NULL, groups_json TEXT NOT NULL, attrs_json TEXT NOT NULL)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Db(e.to_string()))?;
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN access_key TEXT NOT NULL DEFAULT ''")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN secret_key TEXT NOT NULL DEFAULT ''")
            .execute(&self.pool)
            .await;
        sqlx::query("CREATE TABLE IF NOT EXISTS policy_groups (name TEXT PRIMARY KEY, enabled INTEGER NOT NULL, users_json TEXT NOT NULL, rules_json TEXT NOT NULL)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Db(e.to_string()))?;
        sqlx::query("CREATE TABLE IF NOT EXISTS bucket_metadata (bucket TEXT PRIMARY KEY, owner TEXT NOT NULL, labels_json TEXT NOT NULL, attrs_json TEXT NOT NULL)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Db(e.to_string()))?;
        Ok(())
    }

    async fn seed_if_empty(&self, seed: StorageSnapshot) -> Result<(), StorageError> {
        #[cfg(feature = "multi-user")]
        {
            if self.list_users().await?.is_empty() && !seed.users.is_empty() {
                self.save_users(seed.users).await?;
            }
        }
        #[cfg(feature = "policy")]
        {
            if self.list_policy_groups().await?.is_empty() && !seed.policies.is_empty() {
                self.save_policy_groups(seed.policies).await?;
            }
        }
        if self.list_bucket_metadata().await?.is_empty() && !seed.bucket_metadata.is_empty() {
            for m in seed.bucket_metadata {
                self.upsert_bucket_metadata(m).await?;
            }
        }
        Ok(())
    }
}

#[cfg(feature = "multi-user")]
#[async_trait::async_trait]
impl UserStore for SqliteStore {
    async fn list_users(&self) -> Result<Vec<UserRecord>, StorageError> {
        let rows = sqlx::query_as::<_, (String, i64, String, String, String, String)>(
            "SELECT user_id, enabled, groups_json, attrs_json, access_key, secret_key FROM users",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Db(e.to_string()))?;

        let mut out = Vec::with_capacity(rows.len());
        for (user_id, enabled, groups_json, attrs_json, access_key, secret_key) in rows {
            out.push(UserRecord {
                user_id,
                enabled: enabled != 0,
                access_key,
                secret_key,
                groups: serde_json::from_str(&groups_json)
                    .map_err(|e| StorageError::Serde(e.to_string()))?,
                attrs: serde_json::from_str(&attrs_json)
                    .map_err(|e| StorageError::Serde(e.to_string()))?,
            });
        }
        Ok(out)
    }

    async fn save_users(&self, users: Vec<UserRecord>) -> Result<(), StorageError> {
        let current = self.list_users().await?;
        let current_map: std::collections::HashMap<_, _> = current
            .into_iter()
            .map(|u| (u.user_id.clone(), u))
            .collect();
        let new_ids: HashSet<String> = users.iter().map(|u| u.user_id.clone()).collect();

        for u in &users {
            let changed = current_map
                .get(&u.user_id)
                .map(|x| {
                    x.enabled != u.enabled
                        || x.access_key != u.access_key
                        || x.secret_key != u.secret_key
                        || x.groups != u.groups
                        || x.attrs != u.attrs
                })
                .unwrap_or(true);
            if changed {
                let groups_json = serde_json::to_string(&u.groups)
                    .map_err(|e| StorageError::Serde(e.to_string()))?;
                let attrs_json = serde_json::to_string(&u.attrs)
                    .map_err(|e| StorageError::Serde(e.to_string()))?;
                let res = sqlx::query("INSERT INTO users(user_id, enabled, groups_json, attrs_json, access_key, secret_key) VALUES(?,?,?,?,?,?) ON CONFLICT(user_id) DO UPDATE SET enabled=excluded.enabled, groups_json=excluded.groups_json, attrs_json=excluded.attrs_json, access_key=excluded.access_key, secret_key=excluded.secret_key")
                    .bind(&u.user_id)
                    .bind(if u.enabled { 1_i64 } else { 0_i64 })
                    .bind(groups_json)
                    .bind(attrs_json)
                    .bind(&u.access_key)
                    .bind(&u.secret_key)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| StorageError::Db(e.to_string()))?;
                if res.rows_affected() == 0 {
                    return Err(StorageError::Db(format!(
                        "upsert user failed: {}",
                        u.user_id
                    )));
                }
            }
        }

        for old in current_map.keys() {
            if !new_ids.contains(old) {
                let res = sqlx::query("DELETE FROM users WHERE user_id = ?")
                    .bind(old)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| StorageError::Db(e.to_string()))?;
                if res.rows_affected() == 0 {
                    return Err(StorageError::Db(format!("delete user failed: {}", old)));
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "policy")]
#[async_trait::async_trait]
impl PolicyStore for SqliteStore {
    async fn list_policy_groups(&self) -> Result<Vec<PolicyGroup>, StorageError> {
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT name, users_json, rules_json FROM policy_groups",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Db(e.to_string()))?;

        let mut out = Vec::with_capacity(rows.len());
        for (name, users_json, rules_json) in rows {
            out.push(PolicyGroup {
                name,
                users: serde_json::from_str(&users_json)
                    .map_err(|e| StorageError::Serde(e.to_string()))?,
                rules: serde_json::from_str(&rules_json)
                    .map_err(|e| StorageError::Serde(e.to_string()))?,
            });
        }
        Ok(out)
    }

    async fn save_policy_groups(&self, groups: Vec<PolicyGroup>) -> Result<(), StorageError> {
        let current = self.list_policy_groups().await?;
        let current_map: std::collections::HashMap<_, _> =
            current.into_iter().map(|g| (g.name.clone(), g)).collect();
        let new_ids: HashSet<String> = groups.iter().map(|g| g.name.clone()).collect();

        for g in &groups {
            let changed = current_map
                .get(&g.name)
                .map(|x| x.users != g.users || x.rules != g.rules)
                .unwrap_or(true);
            if changed {
                let users_json = serde_json::to_string(&g.users)
                    .map_err(|e| StorageError::Serde(e.to_string()))?;
                let rules_json = serde_json::to_string(&g.rules)
                    .map_err(|e| StorageError::Serde(e.to_string()))?;
                let res = sqlx::query("INSERT INTO policy_groups(name, users_json, rules_json) VALUES(?,?,?) ON CONFLICT(name) DO UPDATE SET users_json=excluded.users_json, rules_json=excluded.rules_json")
                    .bind(&g.name)
                    .bind(users_json)
                    .bind(rules_json)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| StorageError::Db(e.to_string()))?;
                if res.rows_affected() == 0 {
                    return Err(StorageError::Db(format!(
                        "upsert policy group failed: {}",
                        g.name
                    )));
                }
            }
        }

        for old in current_map.keys() {
            if !new_ids.contains(old) {
                let res = sqlx::query("DELETE FROM policy_groups WHERE name = ?")
                    .bind(old)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| StorageError::Db(e.to_string()))?;
                if res.rows_affected() == 0 {
                    return Err(StorageError::Db(format!(
                        "delete policy group failed: {}",
                        old
                    )));
                }
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl BucketMetaStore for SqliteStore {
    async fn list_bucket_metadata(&self) -> Result<Vec<BucketMetadata>, StorageError> {
        let rows = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT bucket, owner, labels_json, attrs_json FROM bucket_metadata",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Db(e.to_string()))?;

        let mut out = Vec::with_capacity(rows.len());
        for (bucket, owner, labels_json, attrs_json) in rows {
            out.push(BucketMetadata {
                bucket,
                owner,
                labels: serde_json::from_str(&labels_json)
                    .map_err(|e| StorageError::Serde(e.to_string()))?,
                attrs: serde_json::from_str(&attrs_json)
                    .map_err(|e| StorageError::Serde(e.to_string()))?,
            });
        }
        Ok(out)
    }

    async fn upsert_bucket_metadata(&self, meta: BucketMetadata) -> Result<(), StorageError> {
        let labels_json =
            serde_json::to_string(&meta.labels).map_err(|e| StorageError::Serde(e.to_string()))?;
        let attrs_json =
            serde_json::to_string(&meta.attrs).map_err(|e| StorageError::Serde(e.to_string()))?;
        let res = sqlx::query("INSERT INTO bucket_metadata(bucket, owner, labels_json, attrs_json) VALUES(?,?,?,?) ON CONFLICT(bucket) DO UPDATE SET owner=excluded.owner, labels_json=excluded.labels_json, attrs_json=excluded.attrs_json")
            .bind(&meta.bucket)
            .bind(&meta.owner)
            .bind(labels_json)
            .bind(attrs_json)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Db(e.to_string()))?;
        if res.rows_affected() == 0 {
            return Err(StorageError::Db(format!(
                "upsert bucket metadata failed: {}",
                meta.bucket
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::SqliteStore;
    use crate::storage::BucketMetaStore;
    #[cfg(feature = "policy")]
    use crate::storage::PolicyStore;
    #[cfg(feature = "multi-user")]
    use crate::storage::UserStore;
    #[cfg(feature = "multi-user")]
    use crate::storage::types::UserRecord;
    use crate::storage::types::{BucketMetadata, StorageSnapshot};
    #[cfg(feature = "policy")]
    use crate::{config::PolicyGroup, config::PolicyRule};

    fn test_dsn(name: &str) -> String {
        let _ = name;
        "sqlite::memory:".to_string()
    }

    #[tokio::test]
    #[cfg(feature = "multi-user")]
    async fn sqlite_user_diff_works() {
        let store = SqliteStore::new(test_dsn("users"), StorageSnapshot::default())
            .await
            .expect("create sqlite store failed");

        store
            .save_users(vec![
                UserRecord {
                    user_id: "alice".to_string(),
                    enabled: true,
                    access_key: "alice-ak".to_string(),
                    secret_key: "alice-sk".to_string(),
                    groups: vec!["g1".to_string()],
                    attrs: HashMap::from([("k".to_string(), "v".to_string())]),
                },
                UserRecord {
                    user_id: "bob".to_string(),
                    enabled: true,
                    access_key: "bob-ak".to_string(),
                    secret_key: "bob-sk".to_string(),
                    groups: vec!["g1".to_string()],
                    attrs: HashMap::new(),
                },
            ])
            .await
            .expect("save users 1 failed");

        store
            .save_users(vec![UserRecord {
                user_id: "alice".to_string(),
                enabled: false,
                access_key: "alice-ak-2".to_string(),
                secret_key: "alice-sk-2".to_string(),
                groups: vec!["g2".to_string()],
                attrs: HashMap::new(),
            }])
            .await
            .expect("save users 2 failed");

        let users = store.list_users().await.expect("list users failed");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].user_id, "alice");
        assert!(!users[0].enabled);
        assert_eq!(users[0].access_key, "alice-ak-2");
        assert_eq!(users[0].secret_key, "alice-sk-2");
        assert_eq!(users[0].groups, vec!["g2".to_string()]);
    }

    #[tokio::test]
    #[cfg(feature = "policy")]
    async fn sqlite_policy_diff_works() {
        let store = SqliteStore::new(test_dsn("policy"), StorageSnapshot::default())
            .await
            .expect("create sqlite store failed");

        store
            .save_policy_groups(vec![
                PolicyGroup {
                    name: "g1".to_string(),
                    users: vec!["alice".to_string()],
                    rules: vec![PolicyRule {
                        bucket: "docs".to_string(),
                        prefix: "a/".to_string(),
                        allow: true,
                        users: vec![],
                    }],
                },
                PolicyGroup {
                    name: "g2".to_string(),
                    users: vec!["bob".to_string()],
                    rules: vec![],
                },
            ])
            .await
            .expect("save policy groups 1 failed");

        store
            .save_policy_groups(vec![PolicyGroup {
                name: "g1".to_string(),
                users: vec!["alice".to_string()],
                rules: vec![PolicyRule {
                    bucket: "docs".to_string(),
                    prefix: "b/".to_string(),
                    allow: false,
                    users: vec![],
                }],
            }])
            .await
            .expect("save policy groups 2 failed");

        let groups = store
            .list_policy_groups()
            .await
            .expect("list policy groups failed");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "g1");
        assert_eq!(groups[0].rules[0].prefix, "b/");
    }

    #[tokio::test]
    async fn sqlite_bucket_meta_upsert_works() {
        let store = SqliteStore::new(test_dsn("bucket-meta"), StorageSnapshot::default())
            .await
            .expect("create sqlite store failed");

        store
            .upsert_bucket_metadata(BucketMetadata {
                bucket: "docs".to_string(),
                owner: "alice".to_string(),
                labels: HashMap::from([("env".to_string(), "dev".to_string())]),
                attrs: HashMap::new(),
            })
            .await
            .expect("upsert 1 failed");

        store
            .upsert_bucket_metadata(BucketMetadata {
                bucket: "docs".to_string(),
                owner: "bob".to_string(),
                labels: HashMap::from([("env".to_string(), "prod".to_string())]),
                attrs: HashMap::from([("region".to_string(), "us-east-1".to_string())]),
            })
            .await
            .expect("upsert 2 failed");

        let list = store
            .list_bucket_metadata()
            .await
            .expect("list bucket metadata failed");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].owner, "bob");
        assert_eq!(list[0].labels.get("env").map(String::as_str), Some("prod"));
        assert_eq!(
            list[0].attrs.get("region").map(String::as_str),
            Some("us-east-1")
        );
    }
}
