use super::response::*;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XMLResponse {
    pub body: String,
}

fn to_xml<T: Serialize>(value: &T) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>{}"#,
        quick_xml::se::to_string(value).unwrap()
    )
}

#[derive(Serialize)]
#[serde(rename = "LocationConstraint")]
struct LocationConstraintXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "$value")]
    location: String,
}

impl From<&GetBucketLocationResponse> for XMLResponse {
    fn from(r: &GetBucketLocationResponse) -> Self {
        let xml = LocationConstraintXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            location: r.location.clone().unwrap_or_default(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "ListAllMyBucketsResult")]
struct ListBucketsXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "Buckets")]
    buckets: BucketsWrapper,
}

#[derive(Serialize)]
struct BucketsWrapper {
    #[serde(rename = "Bucket")]
    bucket: Vec<BucketInfo>,
}

impl From<&ListBucketsResponse> for XMLResponse {
    fn from(r: &ListBucketsResponse) -> Self {
        let xml = ListBucketsXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            buckets: BucketsWrapper { bucket: r.buckets.clone() },
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "ListBucketResult")]
struct ListObjectsXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Contents")]
    contents: Vec<ObjectInfo>,
}

impl From<&ListObjectsV1Response> for XMLResponse {
    fn from(r: &ListObjectsV1Response) -> Self {
        let name = r.objects.first().map(|o| o.bucket.clone()).unwrap_or_default();
        let xml = ListObjectsXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            name,
            contents: r.objects.clone(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "InitiateMultipartUploadResult")]
struct InitiateMultipartXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "Bucket")]
    bucket: String,
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "UploadId")]
    upload_id: String,
}

impl From<&NewMultipartUploadResponse> for XMLResponse {
    fn from(r: &NewMultipartUploadResponse) -> Self {
        let xml = InitiateMultipartXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            bucket: r.bucket.clone().unwrap_or_default(),
            key: r.key.clone().unwrap_or_default(),
            upload_id: r.upload_id.clone().unwrap_or_default(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "AbortMultipartUploadResult")]
struct AbortMultipartXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "UploadId")]
    upload_id: String,
}

impl From<&AbortMultipartUploadResponse> for XMLResponse {
    fn from(r: &AbortMultipartUploadResponse) -> Self {
        let xml = AbortMultipartXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            upload_id: r.upload_id.clone().unwrap_or_default(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "CopyObjectResult")]
struct CopyObjectXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "ETag")]
    etag: String,
    #[serde(rename = "LastModified")]
    last_modified: String,
}

impl From<&CopyObjectResponse> for XMLResponse {
    fn from(r: &CopyObjectResponse) -> Self {
        let obj = r.object.as_ref();
        let xml = CopyObjectXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            etag: obj.and_then(|o| o.etag.clone()).unwrap_or_default(),
            last_modified: obj.and_then(|o| o.last_modified.clone()).unwrap_or_default(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "PutObjectResult")]
struct PutObjectXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "ETag")]
    etag: String,
}

impl From<&PutObjectResponse> for XMLResponse {
    fn from(r: &PutObjectResponse) -> Self {
        let xml = PutObjectXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            etag: r.meta.etag.clone().unwrap_or_default(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "Tagging")]
struct TaggingXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "TagSet")]
    tag_set: TagSetWrapper,
}

#[derive(Serialize)]
struct TagSetWrapper {
    #[serde(rename = "Tag")]
    tags: Vec<TagXml>,
}

#[derive(Serialize)]
struct TagXml {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "Value")]
    value: String,
}

impl From<&GetObjectTaggingResponse> for XMLResponse {
    fn from(r: &GetObjectTaggingResponse) -> Self {
        let tags = r.tags.iter().map(|(k, v)| TagXml { key: k.clone(), value: v.clone() }).collect();
        let xml = TaggingXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            tag_set: TagSetWrapper { tags },
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

impl From<&GetBucketTaggingResponse> for XMLResponse {
    fn from(r: &GetBucketTaggingResponse) -> Self {
        let tags = r.tags.iter().map(|(k, v)| TagXml { key: k.clone(), value: v.clone() }).collect();
        let xml = TaggingXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            tag_set: TagSetWrapper { tags },
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "VersioningConfiguration")]
struct VersioningXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "MfaDelete", skip_serializing_if = "Option::is_none")]
    mfa_delete: Option<String>,
}

impl From<&GetBucketVersioningResponse> for XMLResponse {
    fn from(r: &GetBucketVersioningResponse) -> Self {
        let xml = VersioningXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            status: r.status.clone().unwrap_or_else(|| "Enabled".to_string()),
            mfa_delete: r.mfa_delete.clone(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "NotificationConfiguration")]
struct NotificationXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
}

impl From<&GetBucketNotificationResponse> for XMLResponse {
    fn from(_r: &GetBucketNotificationResponse) -> Self {
        let xml = NotificationXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "ListMultipartUploadsResult")]
struct ListMultipartUploadsXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "Upload")]
    uploads: Vec<MultipartUploadInfo>,
}

impl From<&ListMultipartUploadsResponse> for XMLResponse {
    fn from(r: &ListMultipartUploadsResponse) -> Self {
        let xml = ListMultipartUploadsXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            uploads: r.uploads.clone(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "ListPartsResult")]
struct ListPartsXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "UploadId", skip_serializing_if = "Option::is_none")]
    upload_id: Option<String>,
    #[serde(rename = "Part")]
    parts: Vec<MultipartPartInfo>,
}

impl From<&ListObjectPartsResponse> for XMLResponse {
    fn from(r: &ListObjectPartsResponse) -> Self {
        let xml = ListPartsXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            upload_id: r.upload_id.clone(),
            parts: r.parts.clone(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

impl From<&ListBucketsDoubleSlashResponse> for XMLResponse {
    fn from(r: &ListBucketsDoubleSlashResponse) -> Self {
        let xml = ListBucketsXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            buckets: BucketsWrapper { bucket: r.buckets.clone() },
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

impl From<&ListObjectsV2Response> for XMLResponse {
    fn from(r: &ListObjectsV2Response) -> Self {
        let name = r.objects.first().map(|o| o.bucket.clone()).unwrap_or_default();
        let xml = ListObjectsXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            name,
            contents: r.objects.clone(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

impl From<&ListObjectsV2MResponse> for XMLResponse {
    fn from(r: &ListObjectsV2MResponse) -> Self {
        let name = r.objects.first().map(|o| o.bucket.clone()).unwrap_or_default();
        let xml = ListObjectsXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            name,
            contents: r.objects.clone(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "ListVersionsResult")]
struct ListVersionsXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Contents")]
    contents: Vec<ObjectInfo>,
}

impl From<&ListObjectVersionsResponse> for XMLResponse {
    fn from(r: &ListObjectVersionsResponse) -> Self {
        let name = r.objects.first().map(|o| o.bucket.clone()).unwrap_or_default();
        let xml = ListVersionsXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            name,
            contents: r.objects.clone(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

impl From<&ListObjectVersionsMResponse> for XMLResponse {
    fn from(r: &ListObjectVersionsMResponse) -> Self {
        let name = r.objects.first().map(|o| o.bucket.clone()).unwrap_or_default();
        let xml = ListVersionsXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            name,
            contents: r.objects.clone(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}

#[derive(Serialize)]
#[serde(rename = "CompleteMultipartUploadResult")]
struct CompleteMultipartXml {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "Bucket")]
    bucket: String,
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "ETag")]
    etag: String,
}

impl From<&CompleteMultipartUploadResponse> for XMLResponse {
    fn from(r: &CompleteMultipartUploadResponse) -> Self {
        let obj = r.object.as_ref();
        let xml = CompleteMultipartXml {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/",
            bucket: obj.map(|o| o.bucket.clone()).unwrap_or_default(),
            key: obj.map(|o| o.key.clone()).unwrap_or_default(),
            etag: obj.and_then(|o| o.etag.clone()).unwrap_or_default(),
        };
        XMLResponse { body: to_xml(&xml) }
    }
}
