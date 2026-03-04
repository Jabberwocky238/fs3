use super::response::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XMLResponse {
    pub body: String,
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

impl From<&GetBucketLocationResponse> for XMLResponse {
    fn from(r: &GetBucketLocationResponse) -> Self {
        let loc = r.location.as_deref().unwrap_or("");
        XMLResponse {
            body: format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><LocationConstraint xmlns="http://s3.amazonaws.com/doc/2006-03-01/">{}</LocationConstraint>"#,
                xml_escape(loc)
            )
        }
    }
}

impl From<&ListBucketsResponse> for XMLResponse {
    fn from(r: &ListBucketsResponse) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><Buckets>"#);
        for b in &r.buckets {
            xml.push_str(&format!(
                "<Bucket><Name>{}</Name><CreationDate>{}</CreationDate></Bucket>",
                xml_escape(&b.name),
                b.creation_date.as_deref().unwrap_or("")
            ));
        }
        xml.push_str("</Buckets></ListAllMyBucketsResult>");
        XMLResponse { body: xml }
    }
}

impl From<&ListObjectsV1Response> for XMLResponse {
    fn from(r: &ListObjectsV1Response) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
        if let Some(first) = r.objects.first() {
            xml.push_str(&format!("<Name>{}</Name>", xml_escape(&first.bucket)));
        }
        for o in &r.objects {
            xml.push_str(&format!(
                "<Contents><Key>{}</Key><Size>{}</Size><ETag>{}</ETag><LastModified>{}</LastModified><StorageClass>{}</StorageClass></Contents>",
                xml_escape(&o.key), o.size,
                xml_escape(o.etag.as_deref().unwrap_or("")),
                xml_escape(o.last_modified.as_deref().unwrap_or("")),
                xml_escape(o.storage_class.as_deref().unwrap_or("STANDARD")),
            ));
        }
        xml.push_str("</ListBucketResult>");
        XMLResponse { body: xml }
    }
}

impl From<&NewMultipartUploadResponse> for XMLResponse {
    fn from(r: &NewMultipartUploadResponse) -> Self {
        XMLResponse {
            body: format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><InitiateMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><Bucket>{}</Bucket><Key>{}</Key><UploadId>{}</UploadId></InitiateMultipartUploadResult>"#,
                xml_escape(r.bucket.as_deref().unwrap_or("")),
                xml_escape(r.key.as_deref().unwrap_or("")),
                xml_escape(r.upload_id.as_deref().unwrap_or(""))
            )
        }
    }
}

impl From<&CopyObjectResponse> for XMLResponse {
    fn from(r: &CopyObjectResponse) -> Self {
        let obj = r.object.as_ref();
        XMLResponse {
            body: format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><CopyObjectResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><ETag>{}</ETag><LastModified>{}</LastModified></CopyObjectResult>"#,
                xml_escape(obj.and_then(|o| o.etag.as_deref()).unwrap_or("")),
                xml_escape(obj.and_then(|o| o.last_modified.as_deref()).unwrap_or(""))
            )
        }
    }
}

impl From<&PutObjectResponse> for XMLResponse {
    fn from(r: &PutObjectResponse) -> Self {
        XMLResponse {
            body: format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><PutObjectResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><ETag>{}</ETag></PutObjectResult>"#,
                xml_escape(r.meta.etag.as_deref().unwrap_or(""))
            )
        }
    }
}

impl From<&GetObjectTaggingResponse> for XMLResponse {
    fn from(r: &GetObjectTaggingResponse) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><Tagging xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><TagSet>"#);
        for (k, v) in &r.tags {
            xml.push_str(&format!("<Tag><Key>{}</Key><Value>{}</Value></Tag>", xml_escape(k), xml_escape(v)));
        }
        xml.push_str("</TagSet></Tagging>");
        XMLResponse { body: xml }
    }
}

impl From<&GetBucketTaggingResponse> for XMLResponse {
    fn from(r: &GetBucketTaggingResponse) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><Tagging xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><TagSet>"#);
        for (k, v) in &r.tags {
            xml.push_str(&format!("<Tag><Key>{}</Key><Value>{}</Value></Tag>", xml_escape(k), xml_escape(v)));
        }
        xml.push_str("</TagSet></Tagging>");
        XMLResponse { body: xml }
    }
}

impl From<&GetBucketVersioningResponse> for XMLResponse {
    fn from(r: &GetBucketVersioningResponse) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><VersioningConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
        if let Some(status) = &r.status {
            xml.push_str(&format!("<Status>{}</Status>", xml_escape(status)));
        }
        if let Some(mfa) = &r.mfa_delete {
            xml.push_str(&format!("<MfaDelete>{}</MfaDelete>", xml_escape(mfa)));
        }
        xml.push_str("</VersioningConfiguration>");
        XMLResponse { body: xml }
    }
}

impl From<&GetBucketNotificationResponse> for XMLResponse {
    fn from(_r: &GetBucketNotificationResponse) -> Self {
        XMLResponse {
            body: r#"<?xml version="1.0" encoding="UTF-8"?><NotificationConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/"></NotificationConfiguration>"#.to_string()
        }
    }
}

impl From<&ListMultipartUploadsResponse> for XMLResponse {
    fn from(r: &ListMultipartUploadsResponse) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListMultipartUploadsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
        for u in &r.uploads {
            xml.push_str(&format!(
                "<Upload><Key>{}</Key><UploadId>{}</UploadId></Upload>",
                xml_escape(&u.key), xml_escape(&u.upload_id)
            ));
        }
        xml.push_str("</ListMultipartUploadsResult>");
        XMLResponse { body: xml }
    }
}

impl From<&ListObjectPartsResponse> for XMLResponse {
    fn from(r: &ListObjectPartsResponse) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListPartsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
        if let Some(uid) = &r.upload_id {
            xml.push_str(&format!("<UploadId>{}</UploadId>", xml_escape(uid)));
        }
        for p in &r.parts {
            xml.push_str(&format!(
                "<Part><PartNumber>{}</PartNumber><ETag>{}</ETag><Size>{}</Size></Part>",
                p.part_number, xml_escape(p.etag.as_deref().unwrap_or("")), p.size
            ));
        }
        xml.push_str("</ListPartsResult>");
        XMLResponse { body: xml }
    }
}

impl From<&ListBucketsDoubleSlashResponse> for XMLResponse {
    fn from(r: &ListBucketsDoubleSlashResponse) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><Buckets>"#);
        for b in &r.buckets {
            xml.push_str(&format!(
                "<Bucket><Name>{}</Name><CreationDate>{}</CreationDate></Bucket>",
                xml_escape(&b.name),
                b.creation_date.as_deref().unwrap_or("")
            ));
        }
        xml.push_str("</Buckets></ListAllMyBucketsResult>");
        XMLResponse { body: xml }
    }
}

impl From<&ListObjectsV2Response> for XMLResponse {
    fn from(r: &ListObjectsV2Response) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
        if let Some(first) = r.objects.first() {
            xml.push_str(&format!("<Name>{}</Name>", xml_escape(&first.bucket)));
        }
        for o in &r.objects {
            xml.push_str(&format!(
                "<Contents><Key>{}</Key><Size>{}</Size><ETag>{}</ETag><LastModified>{}</LastModified><StorageClass>{}</StorageClass></Contents>",
                xml_escape(&o.key), o.size,
                xml_escape(o.etag.as_deref().unwrap_or("")),
                xml_escape(o.last_modified.as_deref().unwrap_or("")),
                xml_escape(o.storage_class.as_deref().unwrap_or("STANDARD")),
            ));
        }
        xml.push_str("</ListBucketResult>");
        XMLResponse { body: xml }
    }
}

impl From<&ListObjectsV2MResponse> for XMLResponse {
    fn from(r: &ListObjectsV2MResponse) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
        if let Some(first) = r.objects.first() {
            xml.push_str(&format!("<Name>{}</Name>", xml_escape(&first.bucket)));
        }
        for o in &r.objects {
            xml.push_str(&format!(
                "<Contents><Key>{}</Key><Size>{}</Size><ETag>{}</ETag><LastModified>{}</LastModified><StorageClass>{}</StorageClass></Contents>",
                xml_escape(&o.key), o.size,
                xml_escape(o.etag.as_deref().unwrap_or("")),
                xml_escape(o.last_modified.as_deref().unwrap_or("")),
                xml_escape(o.storage_class.as_deref().unwrap_or("STANDARD")),
            ));
        }
        xml.push_str("</ListBucketResult>");
        XMLResponse { body: xml }
    }
}

impl From<&ListObjectVersionsResponse> for XMLResponse {
    fn from(r: &ListObjectVersionsResponse) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListVersionsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
        if let Some(first) = r.objects.first() {
            xml.push_str(&format!("<Name>{}</Name>", xml_escape(&first.bucket)));
        }
        for o in &r.objects {
            xml.push_str(&format!(
                "<Contents><Key>{}</Key><Size>{}</Size><ETag>{}</ETag><LastModified>{}</LastModified><StorageClass>{}</StorageClass></Contents>",
                xml_escape(&o.key), o.size,
                xml_escape(o.etag.as_deref().unwrap_or("")),
                xml_escape(o.last_modified.as_deref().unwrap_or("")),
                xml_escape(o.storage_class.as_deref().unwrap_or("STANDARD")),
            ));
        }
        xml.push_str("</ListVersionsResult>");
        XMLResponse { body: xml }
    }
}

impl From<&ListObjectVersionsMResponse> for XMLResponse {
    fn from(r: &ListObjectVersionsMResponse) -> Self {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListVersionsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
        if let Some(first) = r.objects.first() {
            xml.push_str(&format!("<Name>{}</Name>", xml_escape(&first.bucket)));
        }
        for o in &r.objects {
            xml.push_str(&format!(
                "<Contents><Key>{}</Key><Size>{}</Size><ETag>{}</ETag><LastModified>{}</LastModified><StorageClass>{}</StorageClass></Contents>",
                xml_escape(&o.key), o.size,
                xml_escape(o.etag.as_deref().unwrap_or("")),
                xml_escape(o.last_modified.as_deref().unwrap_or("")),
                xml_escape(o.storage_class.as_deref().unwrap_or("STANDARD")),
            ));
        }
        xml.push_str("</ListVersionsResult>");
        XMLResponse { body: xml }
    }
}

impl From<&CompleteMultipartUploadResponse> for XMLResponse {
    fn from(r: &CompleteMultipartUploadResponse) -> Self {
        let obj = r.object.as_ref();
        XMLResponse {
            body: format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><CompleteMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><Bucket>{}</Bucket><Key>{}</Key><ETag>{}</ETag></CompleteMultipartUploadResult>"#,
                xml_escape(obj.map(|o| o.bucket.as_str()).unwrap_or("")),
                xml_escape(obj.map(|o| o.key.as_str()).unwrap_or("")),
                xml_escape(obj.and_then(|o| o.etag.as_deref()).unwrap_or(""))
            )
        }
    }
}
