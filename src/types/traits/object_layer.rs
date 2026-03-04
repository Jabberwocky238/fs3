use async_trait::async_trait;
use crate::types::s3::object_layer_types::*;
use crate::types::s3::storage_types::*;
use crate::types::s3::core::BoxByteStream;
use crate::types::errors::*;

#[async_trait]
pub trait ObjectBucketLayer: Send + Sync {
    async fn make_bucket(&self, ctx: &Context, bucket: &str, opts: MakeBucketOptions) -> Result<(), S3Error>;
    async fn get_bucket_info(&self, ctx: &Context, bucket: &str, opts: BucketOptions) -> Result<BucketInfo, S3Error>;
    async fn list_buckets(&self, ctx: &Context, opts: BucketOptions) -> Result<Vec<BucketInfo>, S3Error>;
    async fn delete_bucket(&self, ctx: &Context, bucket: &str, opts: DeleteBucketOptions) -> Result<(), S3Error>;
}

#[async_trait]
pub trait ObjectObjectLayer: Send + Sync {
    async fn get_object_info(&self, ctx: &Context, bucket: &str, object: &str, opts: ObjectOptions) -> Result<ObjectInfo, S3Error>;
    async fn get_object(&self, ctx: &Context, bucket: &str, object: &str, opts: ObjectOptions) -> Result<(ObjectInfo, BoxByteStream), S3Error>;
    async fn put_object(&self, ctx: &Context, bucket: &str, object: &str, data: PutObjReader, opts: ObjectOptions) -> Result<ObjectInfo, S3Error>;
    async fn copy_object(&self, ctx: &Context, src_bucket: &str, src_object: &str, dst_bucket: &str, dst_object: &str, src_info: ObjectInfo, src_opts: ObjectOptions, dst_opts: ObjectOptions) -> Result<ObjectInfo, S3Error>;
    async fn delete_object(&self, ctx: &Context, bucket: &str, object: &str, opts: ObjectOptions) -> Result<ObjectInfo, S3Error>;
}

#[async_trait]
pub trait ObjectMultipartLayer: Send + Sync {
    async fn new_multipart_upload(&self, ctx: &Context, bucket: &str, object: &str, opts: ObjectOptions) -> Result<NewMultipartUploadResult, S3Error>;
    async fn put_object_part(&self, ctx: &Context, bucket: &str, object: &str, upload_id: &str, part_id: u32, data: PutObjReader, opts: ObjectOptions) -> Result<PartInfo, S3Error>;
    async fn complete_multipart_upload(&self, ctx: &Context, bucket: &str, object: &str, upload_id: &str, parts: Vec<CompletePart>, opts: ObjectOptions) -> Result<ObjectInfo, S3Error>;
    async fn abort_multipart_upload(&self, ctx: &Context, bucket: &str, object: &str, upload_id: &str, opts: ObjectOptions) -> Result<(), S3Error>;
}


pub trait ObjectLayer: ObjectBucketLayer + ObjectObjectLayer + ObjectMultipartLayer {}

impl<T> ObjectLayer for T where T: ObjectBucketLayer + ObjectObjectLayer + ObjectMultipartLayer {}