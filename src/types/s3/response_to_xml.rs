use super::response::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XMLResponse {
    pub body: String,
}

impl XMLResponse {
    fn new(body: String) -> Self {
        Self { body }
    }
}

impl From<&S3Response> for Option<XMLResponse> {
    fn from(resp: &S3Response) -> Self {
    match resp {
        // Empty responses (no XML body)
        S3Response::PutBucket(_)
        | S3Response::PutBucketPolicy(_)
        | S3Response::PutBucketTagging(_)
        | S3Response::PutBucketVersioning(_)
        | S3Response::PutBucketNotification(_)
        | S3Response::PutBucketLifecycle(_)
        | S3Response::PutBucketReplicationConfig(_)
        | S3Response::PutBucketEncryption(_)
        | S3Response::PutBucketObjectLockConfig(_)
        | S3Response::PutBucketAcl(_)
        | S3Response::PutBucketCors(_)
        | S3Response::PutObjectAcl(_)
        | S3Response::PutObjectTagging(_)
        | S3Response::PutObjectRetention(_)
        | S3Response::PutObjectLegalHold(_)
        | S3Response::ResetBucketReplicationStart(_)
        | S3Response::ResetBucketReplicationStatus(_)
        | S3Response::PostRestoreObject(_)
        | S3Response::HeadBucket(_)
        | S3Response::DeleteBucket(_)
        | S3Response::DeleteBucketPolicy(_)
        | S3Response::DeleteBucketTagging(_)
        | S3Response::DeleteBucketEncryption(_)
        | S3Response::DeleteBucketLifecycle(_)
        | S3Response::DeleteBucketReplication(_)
        | S3Response::DeleteBucketWebsite(_)
        | S3Response::DeleteBucketCors(_)
        | S3Response::DeleteObject(_)
        | S3Response::DeleteObjectTagging(_) => None,

        S3Response::GetBucketLocation(r) => Some(XMLResponse::new(get_bucket_location_xml(r))),
        S3Response::ListBuckets(r) => Some(XMLResponse::new(list_buckets_xml(&r.buckets))),
        S3Response::ListBucketsDoubleSlash(r) => Some(XMLResponse::new(list_buckets_xml(&r.buckets))),
        S3Response::ListObjectsV1(r) => Some(XMLResponse::new(list_objects_xml("ListBucketResult", &r.objects))),
        S3Response::ListObjectsV2(r) => Some(XMLResponse::new(list_objects_xml("ListBucketResult", &r.objects))),
        S3Response::ListObjectsV2M(r) => Some(XMLResponse::new(list_objects_xml("ListBucketResult", &r.objects))),
        S3Response::ListObjectVersions(r) => Some(XMLResponse::new(list_objects_xml("ListVersionsResult", &r.objects))),
        S3Response::ListObjectVersionsM(r) => Some(XMLResponse::new(list_objects_xml("ListVersionsResult", &r.objects))),
        S3Response::ListMultipartUploads(r) => Some(XMLResponse::new(list_multipart_uploads_xml(&r.uploads))),
        S3Response::ListObjectParts(r) => Some(XMLResponse::new(list_object_parts_xml(r))),
        S3Response::NewMultipartUpload(r) => Some(XMLResponse::new(new_multipart_upload_xml(r))),
        S3Response::AbortMultipartUpload(_) => Some(XMLResponse::new(abort_multipart_upload_xml())),
        S3Response::CompleteMultipartUpload(r) => Some(XMLResponse::new(complete_multipart_upload_xml(r))),

        // Passthrough XML - TODO: implement proper structure
        S3Response::GetBucketLifecycle(_) => None,
        S3Response::GetBucketEncryption(_) => None,
        S3Response::GetBucketObjectLockConfig(_) => None,
        S3Response::GetBucketReplicationConfig(_) => None,
        S3Response::GetBucketVersioning(_) => None,
        S3Response::GetBucketNotification(_) => Some(XMLResponse::new(empty_notification_xml())),
        S3Response::GetBucketAcl(_) => None,
        S3Response::GetBucketCors(_) => None,
        S3Response::GetBucketWebsite(_) => None,
        S3Response::GetBucketAccelerate(_) => None,
        S3Response::GetBucketRequestPayment(_) => None,
        S3Response::GetBucketLogging(_) => None,
        S3Response::GetBucketTagging(r) => Some(XMLResponse::new(tags_to_tagging_xml(&r.tags))),
        S3Response::GetObjectAcl(_) => None,
        S3Response::GetObjectTagging(r) => Some(XMLResponse::new(tags_to_tagging_xml(&r.tags))),
        S3Response::GetObjectRetention(_) => None,
        S3Response::GetObjectLegalHold(_) => None,

        S3Response::CopyObject(r) => Some(XMLResponse::new(copy_object_xml(r))),
        S3Response::PutObject(r) => Some(XMLResponse::new(put_object_xml(r))),
        S3Response::HeadObject(_) => None,
        S3Response::GetObject(_) => None,
        S3Response::GetObjectLambda(_) => None,
        S3Response::SelectObjectContent(_) => None,
        S3Response::GetBucketPolicy(_) => None,

        _ => None,
    }
}
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn get_bucket_location_xml(r: &GetBucketLocationResponse) -> String {
    let loc = r.location.as_deref().unwrap_or("");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><LocationConstraint xmlns="http://s3.amazonaws.com/doc/2006-03-01/">{}</LocationConstraint>"#,
        xml_escape(loc)
    )
}

fn list_buckets_xml(buckets: &[BucketInfo]) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><Buckets>"#);
    for b in buckets {
        xml.push_str(&format!(
            "<Bucket><Name>{}</Name><CreationDate>{}</CreationDate></Bucket>",
            xml_escape(&b.name),
            b.creation_date.as_deref().unwrap_or("")
        ));
    }
    xml.push_str("</Buckets></ListAllMyBucketsResult>");
    xml
}

fn list_objects_xml(root: &str, objects: &[ObjectInfo]) -> String {
    let mut xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?><{} xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#, root);
    if let Some(first) = objects.first() {
        xml.push_str(&format!("<Name>{}</Name>", xml_escape(&first.bucket)));
    }
    for o in objects {
        xml.push_str(&format!(
            "<Contents><Key>{}</Key><Size>{}</Size><ETag>{}</ETag><LastModified>{}</LastModified><StorageClass>{}</StorageClass></Contents>",
            xml_escape(&o.key), o.size,
            xml_escape(o.etag.as_deref().unwrap_or("")),
            xml_escape(o.last_modified.as_deref().unwrap_or("")),
            xml_escape(o.storage_class.as_deref().unwrap_or("STANDARD")),
        ));
    }
    xml.push_str(&format!("</{}>", root));
    xml
}

fn list_multipart_uploads_xml(uploads: &[MultipartUploadInfo]) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListMultipartUploadsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
    for u in uploads {
        xml.push_str(&format!(
            "<Upload><Key>{}</Key><UploadId>{}</UploadId></Upload>",
            xml_escape(&u.key), xml_escape(&u.upload_id)
        ));
    }
    xml.push_str("</ListMultipartUploadsResult>");
    xml
}

fn list_object_parts_xml(r: &ListObjectPartsResponse) -> String {
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
    xml
}

fn new_multipart_upload_xml(r: &NewMultipartUploadResponse) -> String {
    let bucket = r.bucket.as_deref().unwrap_or("");
    let key = r.key.as_deref().unwrap_or("");
    let uid = r.upload_id.as_deref().unwrap_or("");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><InitiateMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><Bucket>{}</Bucket><Key>{}</Key><UploadId>{}</UploadId></InitiateMultipartUploadResult>"#,
        xml_escape(bucket), xml_escape(key), xml_escape(uid)
    )
}

fn abort_multipart_upload_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?><AbortMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"></AbortMultipartUploadResult>"#.to_string()
}

fn complete_multipart_upload_xml(r: &CompleteMultipartUploadResponse) -> String {
    let obj = r.object.as_ref();
    let etag_raw = obj.and_then(|o| o.etag.as_deref()).unwrap_or("");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><CompleteMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><Bucket>{}</Bucket><Key>{}</Key><ETag>{}</ETag></CompleteMultipartUploadResult>"#,
        xml_escape(obj.map(|o| o.bucket.as_str()).unwrap_or("")),
        xml_escape(obj.map(|o| o.key.as_str()).unwrap_or("")),
        xml_escape(etag_raw)
    )
}

fn copy_object_xml(r: &CopyObjectResponse) -> String {
    let obj = r.object.as_ref();
    let etag = obj.and_then(|o| o.etag.as_deref()).unwrap_or("");
    let last_modified = obj.and_then(|o| o.last_modified.as_deref()).unwrap_or("");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><CopyObjectResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><ETag>{}</ETag><LastModified>{}</LastModified></CopyObjectResult>"#,
        xml_escape(etag), xml_escape(last_modified)
    )
}

fn put_object_xml(r: &PutObjectResponse) -> String {
    let etag = r.meta.etag.as_deref().unwrap_or("");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><PutObjectResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><ETag>{}</ETag></PutObjectResult>"#,
        xml_escape(etag)
    )
}

fn empty_tagging_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?><Tagging xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><TagSet/></Tagging>"#.to_string()
}

fn tags_to_tagging_xml(tags: &HashMap<String, String>) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><Tagging xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><TagSet>"#);
    for (k, v) in tags {
        xml.push_str(&format!("<Tag><Key>{}</Key><Value>{}</Value></Tag>", xml_escape(k), xml_escape(v)));
    }
    xml.push_str("</TagSet></Tagging>");
    xml
}

fn empty_notification_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?><NotificationConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/"></NotificationConfiguration>"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("hello"), "hello");
        assert_eq!(xml_escape("<test>"), "&lt;test&gt;");
        assert_eq!(xml_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn test_get_bucket_location_xml() {
        let resp = GetBucketLocationResponse {
            location: Some("us-east-1".to_string()),
            ..Default::default()
        };
        let xml = get_bucket_location_xml(&resp);
        assert!(xml.contains("<LocationConstraint"));
        assert!(xml.contains("us-east-1"));
    }

    #[test]
    fn test_list_buckets_xml() {
        let buckets = vec![
            BucketInfo {
                name: "bucket1".to_string(),
                creation_date: Some("2024-01-01T00:00:00Z".to_string()),
            },
        ];
        let xml = list_buckets_xml(&buckets);
        assert!(xml.contains("<ListAllMyBucketsResult"));
        assert!(xml.contains("<Name>bucket1</Name>"));
    }

    #[test]
    fn test_list_objects_xml() {
        let objects = vec![
            ObjectInfo {
                bucket: "test-bucket".to_string(),
                key: "file.txt".to_string(),
                size: 100,
                etag: Some("abc123".to_string()),
                last_modified: Some("2024-01-01T00:00:00Z".to_string()),
                storage_class: Some("STANDARD".to_string()),
            },
        ];
        let xml = list_objects_xml("ListBucketResult", &objects);
        assert!(xml.contains("<Name>test-bucket</Name>"));
        assert!(xml.contains("<Key>file.txt</Key>"));
        assert!(xml.contains("<Size>100</Size>"));
    }

    #[test]
    fn test_new_multipart_upload_xml() {
        let resp = NewMultipartUploadResponse {
            bucket: Some("test-bucket".to_string()),
            key: Some("test-key".to_string()),
            upload_id: Some("upload123".to_string()),
            ..Default::default()
        };
        let xml = new_multipart_upload_xml(&resp);
        assert!(xml.contains("<Bucket>test-bucket</Bucket>"));
        assert!(xml.contains("<Key>test-key</Key>"));
        assert!(xml.contains("<UploadId>upload123</UploadId>"));
    }
}
