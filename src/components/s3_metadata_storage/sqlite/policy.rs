use async_trait::async_trait;

use crate::types::errors::S3MetadataStorageError;
use crate::types::traits::s3_metadata_storage::S3MetadataStoragePolicy;

use super::SqliteMetadataStorage;

type Result<T> = std::result::Result<T, S3MetadataStorageError>;

#[async_trait]
impl S3MetadataStoragePolicy for SqliteMetadataStorage {
    async fn store_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<()> {
        sqlx::query("INSERT OR REPLACE INTO bucket_policies (bucket, policy_json) VALUES (?, ?)")
            .bind(bucket).bind(policy_json)
            .execute(&self.pool).await?;
        Ok(())
    }

    async fn load_bucket_policy(&self, bucket: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT policy_json FROM bucket_policies WHERE bucket = ?")
            .bind(bucket).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| r.0))
    }

    async fn delete_bucket_policy(&self, bucket: &str) -> Result<()> {
        sqlx::query("DELETE FROM bucket_policies WHERE bucket = ?")
            .bind(bucket).execute(&self.pool).await?;
        Ok(())
    }

    async fn store_iam_policy(&self, policy_name: &str, policy_json: &str) -> Result<()> {
        sqlx::query("INSERT OR REPLACE INTO iam_policies (policy_name, policy_json) VALUES (?, ?)")
            .bind(policy_name).bind(policy_json)
            .execute(&self.pool).await?;
        Ok(())
    }

    async fn load_iam_policy(&self, policy_name: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT policy_json FROM iam_policies WHERE policy_name = ?")
            .bind(policy_name).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| r.0))
    }

    async fn delete_iam_policy(&self, policy_name: &str) -> Result<()> {
        sqlx::query("DELETE FROM iam_policies WHERE policy_name = ?")
            .bind(policy_name).execute(&self.pool).await?;
        Ok(())
    }

    async fn list_iam_policies(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT policy_name FROM iam_policies")
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    async fn store_user_policy_mapping(&self, identity: &str, policy_names: &str) -> Result<()> {
        sqlx::query("INSERT OR REPLACE INTO user_policy_mappings (identity, policy_names) VALUES (?, ?)")
            .bind(identity).bind(policy_names)
            .execute(&self.pool).await?;
        Ok(())
    }

    async fn load_user_policy_mapping(&self, identity: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT policy_names FROM user_policy_mappings WHERE identity = ?")
            .bind(identity).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| r.0))
    }

    async fn delete_user_policy_mapping(&self, identity: &str) -> Result<()> {
        sqlx::query("DELETE FROM user_policy_mappings WHERE identity = ?")
            .bind(identity).execute(&self.pool).await?;
        Ok(())
    }

    async fn store_group_policy_mapping(&self, group: &str, policy_names: &str) -> Result<()> {
        sqlx::query("INSERT OR REPLACE INTO group_policy_mappings (group_name, policy_names) VALUES (?, ?)")
            .bind(group).bind(policy_names)
            .execute(&self.pool).await?;
        Ok(())
    }

    async fn load_group_policy_mapping(&self, group: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT policy_names FROM group_policy_mappings WHERE group_name = ?")
            .bind(group).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| r.0))
    }

    async fn delete_group_policy_mapping(&self, group: &str) -> Result<()> {
        sqlx::query("DELETE FROM group_policy_mappings WHERE group_name = ?")
            .bind(group).execute(&self.pool).await?;
        Ok(())
    }
}
