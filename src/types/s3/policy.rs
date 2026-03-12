use std::fmt;

/// S3 策略动作枚举，覆盖所有 S3 API 操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum S3Action {
    // --- Object actions ---
    GetObject,
    HeadObject,
    PutObject,
    DeleteObject,
    CopyObject,
    GetObjectAttributes,
    GetObjectTagging,
    PutObjectTagging,
    DeleteObjectTagging,
    GetObjectRetention,
    PutObjectRetention,
    GetObjectLegalHold,
    PutObjectLegalHold,
    GetObjectAcl,
    PutObjectAcl,
    RestoreObject,
    SelectObjectContent,
    ListMultipartUploadParts,
    AbortMultipartUpload,
    DeleteObjectVersion,
    GetObjectVersion,
    GetObjectVersionAttributes,
    ReplicateObject,
    ReplicateDelete,
    PutObjectFanOut,

    // --- Bucket actions ---
    CreateBucket,
    DeleteBucket,
    ForceDeleteBucket,
    HeadBucket,
    ListAllMyBuckets,
    ListBucket,
    ListBucketVersions,
    ListBucketMultipartUploads,
    GetBucketLocation,
    GetBucketPolicy,
    PutBucketPolicy,
    DeleteBucketPolicy,
    GetBucketPolicyStatus,
    GetBucketVersioning,
    PutBucketVersioning,
    GetBucketEncryption,
    PutBucketEncryption,
    GetBucketLifecycle,
    PutBucketLifecycle,
    GetBucketNotification,
    PutBucketNotification,
    GetBucketTagging,
    PutBucketTagging,
    GetBucketObjectLockConfiguration,
    PutBucketObjectLockConfiguration,
    GetReplicationConfiguration,
    PutReplicationConfiguration,
    GetBucketCors,
    PutBucketCors,
    DeleteBucketCors,
    GetBucketAcl,
    PutBucketAcl,
    GetBucketWebsite,
    PutBucketWebsite,
    DeleteBucketWebsite,
    GetBucketAccelerate,
    GetBucketRequestPayment,
    GetBucketLogging,
    ResetBucketReplicationState,
}

impl S3Action {
    /// 返回 "s3:XxxYyy" 格式的动作字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GetObject => "s3:GetObject",
            Self::HeadObject => "s3:GetObject",
            Self::PutObject => "s3:PutObject",
            Self::DeleteObject => "s3:DeleteObject",
            Self::CopyObject => "s3:PutObject",
            Self::GetObjectAttributes => "s3:GetObject",
            Self::GetObjectTagging => "s3:GetObjectTagging",
            Self::PutObjectTagging => "s3:PutObjectTagging",
            Self::DeleteObjectTagging => "s3:DeleteObjectTagging",
            Self::GetObjectRetention => "s3:GetObjectRetention",
            Self::PutObjectRetention => "s3:PutObjectRetention",
            Self::GetObjectLegalHold => "s3:GetObjectLegalHold",
            Self::PutObjectLegalHold => "s3:PutObjectLegalHold",
            Self::GetObjectAcl => "s3:GetObjectAcl",
            Self::PutObjectAcl => "s3:PutObjectAcl",
            Self::RestoreObject => "s3:RestoreObject",
            Self::SelectObjectContent => "s3:GetObject",
            Self::ListMultipartUploadParts => "s3:ListMultipartUploadParts",
            Self::AbortMultipartUpload => "s3:AbortMultipartUpload",
            Self::DeleteObjectVersion => "s3:DeleteObjectVersion",
            Self::GetObjectVersion => "s3:GetObjectVersion",
            Self::GetObjectVersionAttributes => "s3:GetObjectVersionAttributes",
            Self::ReplicateObject => "s3:ReplicateObject",
            Self::ReplicateDelete => "s3:ReplicateDelete",
            Self::PutObjectFanOut => "s3:PutObject",
            Self::CreateBucket => "s3:CreateBucket",
            Self::DeleteBucket => "s3:DeleteBucket",
            Self::ForceDeleteBucket => "s3:ForceDeleteBucket",
            Self::HeadBucket => "s3:HeadBucket",
            Self::ListAllMyBuckets => "s3:ListAllMyBuckets",
            Self::ListBucket => "s3:ListBucket",
            Self::ListBucketVersions => "s3:ListBucketVersions",
            Self::ListBucketMultipartUploads => "s3:ListBucketMultipartUploads",
            Self::GetBucketLocation => "s3:GetBucketLocation",
            Self::GetBucketPolicy => "s3:GetBucketPolicy",
            Self::PutBucketPolicy => "s3:PutBucketPolicy",
            Self::DeleteBucketPolicy => "s3:DeleteBucketPolicy",
            Self::GetBucketPolicyStatus => "s3:GetBucketPolicyStatus",
            Self::GetBucketVersioning => "s3:GetBucketVersioning",
            Self::PutBucketVersioning => "s3:PutBucketVersioning",
            Self::GetBucketEncryption => "s3:GetEncryptionConfiguration",
            Self::PutBucketEncryption => "s3:PutEncryptionConfiguration",
            Self::GetBucketLifecycle => "s3:GetLifecycleConfiguration",
            Self::PutBucketLifecycle => "s3:PutLifecycleConfiguration",
            Self::GetBucketNotification => "s3:GetBucketNotification",
            Self::PutBucketNotification => "s3:PutBucketNotification",
            Self::GetBucketTagging => "s3:GetBucketTagging",
            Self::PutBucketTagging => "s3:PutBucketTagging",
            Self::GetBucketObjectLockConfiguration => "s3:GetBucketObjectLockConfiguration",
            Self::PutBucketObjectLockConfiguration => "s3:PutBucketObjectLockConfiguration",
            Self::GetReplicationConfiguration => "s3:GetReplicationConfiguration",
            Self::PutReplicationConfiguration => "s3:PutReplicationConfiguration",
            Self::GetBucketCors => "s3:GetBucketCors",
            Self::PutBucketCors => "s3:PutBucketCors",
            Self::DeleteBucketCors => "s3:PutBucketCors",
            Self::GetBucketAcl => "s3:GetBucketAcl",
            Self::PutBucketAcl => "s3:PutBucketAcl",
            Self::GetBucketWebsite => "s3:GetBucketWebsite",
            Self::PutBucketWebsite => "s3:PutBucketWebsite",
            Self::DeleteBucketWebsite => "s3:DeleteBucketWebsite",
            Self::GetBucketAccelerate => "s3:GetAccelerateConfiguration",
            Self::GetBucketRequestPayment => "s3:GetBucketRequestPayment",
            Self::GetBucketLogging => "s3:GetBucketLogging",
            Self::ResetBucketReplicationState => "s3:ResetBucketReplicationState",
        }
    }

    /// 是否为对象级动作
    pub fn is_object_action(&self) -> bool {
        matches!(
            self,
            Self::GetObject
                | Self::HeadObject
                | Self::PutObject
                | Self::DeleteObject
                | Self::CopyObject
                | Self::GetObjectAttributes
                | Self::GetObjectTagging
                | Self::PutObjectTagging
                | Self::DeleteObjectTagging
                | Self::GetObjectRetention
                | Self::PutObjectRetention
                | Self::GetObjectLegalHold
                | Self::PutObjectLegalHold
                | Self::GetObjectAcl
                | Self::PutObjectAcl
                | Self::RestoreObject
                | Self::SelectObjectContent
                | Self::ListMultipartUploadParts
                | Self::AbortMultipartUpload
                | Self::DeleteObjectVersion
                | Self::GetObjectVersion
                | Self::GetObjectVersionAttributes
                | Self::ReplicateObject
                | Self::ReplicateDelete
                | Self::PutObjectFanOut
        )
    }

    /// 是否为桶级动作
    pub fn is_bucket_action(&self) -> bool {
        !self.is_object_action()
    }
}

impl fmt::Display for S3Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
