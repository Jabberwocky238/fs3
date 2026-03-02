use async_trait::async_trait;

use crate::types::s3::core::*;
use crate::types::errors::S3MetadataStorageError;

type Result<T> = std::result::Result<T, S3MetadataStorageError>;

/// Trait for persisting S3 metadata (buckets, objects, multipart uploads).
/// Implementations handle storage to databases, files, etc.
/// This does NOT handle object data — only metadata.
#[async_trait]
pub trait S3MetadataStorageBucket {
    async fn store_bucket(&self, bucket: &S3Bucket) -> Result<()>;
    async fn load_bucket(&self, name: &str) -> Result<Option<S3Bucket>>;
    async fn list_buckets(&self) -> Result<Vec<S3Bucket>>;
    async fn delete_bucket(&self, name: &str) -> Result<()>;
    async fn store_bucket_metadata(&self, bucket: &str, metadata: &BucketMetadataBundle) -> Result<()>;
    async fn load_bucket_metadata(&self, bucket: &str) -> Result<Option<BucketMetadataBundle>>;
}

#[async_trait]
pub trait S3MetadataStorageObject {
    async fn store_object_meta(&self, obj: &S3Object) -> Result<()>;
    async fn load_object_meta(&self, bucket: &str, key: &str) -> Result<Option<S3Object>>;
    async fn delete_object_meta(&self, bucket: &str, key: &str) -> Result<()>;
    async fn list_objects(&self, bucket: &str, options: &ListOptions) -> Result<ObjectListPage>;
}

#[async_trait]
pub trait S3MetadataStorageMultipart {
    async fn store_multipart(&self, upload: &MultipartUpload) -> Result<()>;
    async fn load_multipart(&self, upload_id: &str) -> Result<Option<MultipartUpload>>;
    async fn delete_multipart(&self, upload_id: &str) -> Result<()>;
    async fn store_uploaded_part(&self, upload_id: &str, part: &UploadedPart) -> Result<()>;
    async fn list_uploaded_parts(&self, upload_id: &str) -> Result<Vec<UploadedPart>>;
    async fn list_multipart_uploads(&self, bucket: &str) -> Result<Vec<MultipartUpload>>;
}

/// 策略元数据存储：IAM 用户/组策略映射 + 桶策略持久化
#[async_trait]
pub trait S3MetadataStoragePolicy {
    /// 存储桶策略文档（JSON）
    async fn store_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<()>;
    /// 加载桶策略文档
    async fn load_bucket_policy(&self, bucket: &str) -> Result<Option<String>>;
    /// 删除桶策略
    async fn delete_bucket_policy(&self, bucket: &str) -> Result<()>;

    /// 存储 IAM 策略文档（按策略名）
    async fn store_iam_policy(&self, policy_name: &str, policy_json: &str) -> Result<()>;
    /// 加载 IAM 策略文档
    async fn load_iam_policy(&self, policy_name: &str) -> Result<Option<String>>;
    /// 删除 IAM 策略文档
    async fn delete_iam_policy(&self, policy_name: &str) -> Result<()>;
    /// 列出所有 IAM 策略名
    async fn list_iam_policies(&self) -> Result<Vec<String>>;

    /// 绑定用户到策略（逗号分隔的策略名）
    async fn store_user_policy_mapping(&self, identity: &str, policy_names: &str) -> Result<()>;
    /// 加载用户绑定的策略名
    async fn load_user_policy_mapping(&self, identity: &str) -> Result<Option<String>>;
    /// 删除用户策略绑定
    async fn delete_user_policy_mapping(&self, identity: &str) -> Result<()>;

    /// 绑定组到策略
    async fn store_group_policy_mapping(&self, group: &str, policy_names: &str) -> Result<()>;
    /// 加载组绑定的策略名
    async fn load_group_policy_mapping(&self, group: &str) -> Result<Option<String>>;
    /// 删除组策略绑定
    async fn delete_group_policy_mapping(&self, group: &str) -> Result<()>;
}

pub trait S3MetadataStorage:
    S3MetadataStorageBucket + S3MetadataStorageObject + S3MetadataStorageMultipart + S3MetadataStoragePolicy
{
}

impl<T> S3MetadataStorage for T
where
    T: S3MetadataStorageBucket + S3MetadataStorageObject + S3MetadataStorageMultipart + S3MetadataStoragePolicy,
{
}
