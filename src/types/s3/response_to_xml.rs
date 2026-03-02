use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use super::response::{BucketInfo, ResponseMeta};
use super::response::S3Response;
use super::response::ObjectInfo;

impl IntoResponse for S3Response {
    fn into_response(self) -> Response {
        match self {
            // Empty 200
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
            | S3Response::PostRestoreObject(_) => StatusCode::OK.into_response(),

            S3Response::HeadBucket(r) => {
                let sc = StatusCode::from_u16(r.meta.status_code).unwrap_or(StatusCode::OK);
                (sc, "").into_response()
            }

            // Empty 204
            S3Response::DeleteBucket(_)
            | S3Response::DeleteBucketPolicy(_)
            | S3Response::DeleteBucketTagging(_)
            | S3Response::DeleteBucketEncryption(_)
            | S3Response::DeleteBucketLifecycle(_)
            | S3Response::DeleteBucketReplication(_)
            | S3Response::DeleteBucketWebsite(_)
            | S3Response::DeleteBucketCors(_)
            | S3Response::DeleteObject(_)
            | S3Response::DeleteObjectTagging(_)
            | S3Response::AbortMultipartUpload(_) => StatusCode::NO_CONTENT.into_response(),

            // Raw bytes
            S3Response::GetObject(r) => raw_response(r.meta.status_code, &r.meta, r.body),
            S3Response::GetObjectLambda(r) => raw_response(r.meta.status_code, &r.meta, r.body),
            S3Response::SelectObjectContent(r) => raw_response(200, &r.meta, r.payload),

            // Raw JSON text (GetBucketPolicy returns JSON, not XML)
            S3Response::GetBucketPolicy(r) => {
                let body = r.json.unwrap_or_default();
                (StatusCode::OK, [("content-type", "application/json")], body).into_response()
            }

            // Passthrough XML from engine
            S3Response::GetBucketLifecycle(r) => xml_passthrough(&r.xml, "LifecycleConfiguration"),
            S3Response::GetBucketEncryption(r) => xml_passthrough(&r.xml, "ServerSideEncryptionConfiguration"),
            S3Response::GetBucketObjectLockConfig(r) => xml_passthrough(&r.xml, "ObjectLockConfiguration"),
            S3Response::GetBucketReplicationConfig(r) => xml_passthrough(&r.xml, "ReplicationConfiguration"),
            S3Response::GetBucketVersioning(r) => xml_passthrough(&r.xml, "VersioningConfiguration"),
            S3Response::GetBucketNotification(r) => xml_passthrough(&r.xml, "NotificationConfiguration"),
            S3Response::GetBucketAcl(r) => xml_passthrough(&r.xml, "AccessControlPolicy"),
            S3Response::GetBucketCors(r) => xml_passthrough(&r.xml, "CORSConfiguration"),
            S3Response::GetBucketWebsite(r) => xml_passthrough(&r.xml, "WebsiteConfiguration"),
            S3Response::GetBucketAccelerate(r) => xml_passthrough(&r.xml, "AccelerateConfiguration"),
            S3Response::GetBucketRequestPayment(r) => xml_passthrough(&r.xml, "RequestPaymentConfiguration"),
            S3Response::GetBucketLogging(r) => xml_passthrough(&r.xml, "BucketLoggingStatus"),
            S3Response::GetBucketTagging(r) => xml_passthrough(&r.xml, "Tagging"),
            S3Response::GetObjectAcl(r) => xml_passthrough(&r.xml, "AccessControlPolicy"),
            S3Response::GetObjectTagging(r) => xml_passthrough(&r.xml, "Tagging"),
            S3Response::GetObjectRetention(r) => xml_passthrough(&r.xml, "Retention"),
            S3Response::GetObjectLegalHold(r) => xml_passthrough(&r.xml, "LegalHold"),

            // Built XML responses
            S3Response::GetBucketLocation(r) => {
                let loc = r.location.as_deref().unwrap_or("");
                xml_response(&format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?><LocationConstraint xmlns="http://s3.amazonaws.com/doc/2006-03-01/">{}</LocationConstraint>"#,
                    xml_escape(loc)
                ))
            }

            S3Response::ListBuckets(r) => xml_response(&list_buckets_xml(&r.buckets)),
            S3Response::ListBucketsDoubleSlash(r) => xml_response(&list_buckets_xml(&r.buckets)),

            S3Response::ListObjectsV1(r) => xml_response(&list_objects_xml("ListBucketResult", &r.objects)),
            S3Response::ListObjectsV2(r) => xml_response(&list_objects_xml("ListBucketResult", &r.objects)),
            S3Response::ListObjectsV2M(r) => xml_response(&list_objects_xml("ListBucketResult", &r.objects)),
            S3Response::ListObjectVersions(r) => xml_response(&list_objects_xml("ListVersionsResult", &r.objects)),
            S3Response::ListObjectVersionsM(r) => xml_response(&list_objects_xml("ListVersionsResult", &r.objects)),

            S3Response::ListMultipartUploads(r) => {
                let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListMultipartUploadsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
                for u in &r.uploads {
                    xml.push_str(&format!(
                        "<Upload><Key>{}</Key><UploadId>{}</UploadId></Upload>",
                        xml_escape(&u.key), xml_escape(&u.upload_id)
                    ));
                }
                xml.push_str("</ListMultipartUploadsResult>");
                xml_response(&xml)
            }

            S3Response::ListObjectParts(r) => {
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
                xml_response(&xml)
            }

            S3Response::NewMultipartUpload(r) => {
                let uid = r.upload_id.as_deref().unwrap_or("");
                xml_response(&format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?><InitiateMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><UploadId>{}</UploadId></InitiateMultipartUploadResult>"#,
                    xml_escape(uid)
                ))
            }

            S3Response::CompleteMultipartUpload(r) => {
                let obj = r.object.as_ref();
                xml_response(&format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?><CompleteMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><Bucket>{}</Bucket><Key>{}</Key><ETag>{}</ETag></CompleteMultipartUploadResult>"#,
                    xml_escape(obj.map(|o| o.bucket.as_str()).unwrap_or("")),
                    xml_escape(obj.map(|o| o.key.as_str()).unwrap_or("")),
                    xml_escape(obj.and_then(|o| o.etag.as_deref()).unwrap_or("")),
                ))
            }

            S3Response::CopyObject(r) => {
                let etag = r.object.as_ref().and_then(|o| o.etag.as_deref()).unwrap_or("");
                let lm = r.object.as_ref().and_then(|o| o.last_modified.as_deref()).unwrap_or("");
                xml_response(&format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?><CopyObjectResult><ETag>{}</ETag><LastModified>{}</LastModified></CopyObjectResult>"#,
                    xml_escape(etag), xml_escape(lm)
                ))
            }

            S3Response::CopyObjectPart(r) => {
                let etag = r.part.as_ref().and_then(|p| p.etag.as_deref()).unwrap_or("");
                xml_response(&format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?><CopyPartResult><ETag>{}</ETag></CopyPartResult>"#,
                    xml_escape(etag)
                ))
            }

            S3Response::PutObject(r) => {
                let mut resp = StatusCode::OK.into_response();
                if let Some(obj) = &r.object {
                    if let Some(etag) = &obj.etag {
                        if let Ok(v) = etag.parse() {
                            resp.headers_mut().insert("etag", v);
                        }
                    }
                }
                resp
            }

            S3Response::PutObjectPart(r) => {
                let mut resp = StatusCode::OK.into_response();
                if let Some(p) = &r.part {
                    if let Some(etag) = &p.etag {
                        if let Ok(v) = etag.parse() {
                            resp.headers_mut().insert("etag", v);
                        }
                    }
                }
                resp
            }

            S3Response::HeadObject(r) => {
                let mut builder = Response::builder().status(
                    StatusCode::from_u16(r.meta.status_code).unwrap_or(StatusCode::OK)
                );
                for (k, v) in &r.headers {
                    if let Ok(hv) = v.parse::<axum::http::HeaderValue>() {
                        builder = builder.header(k.as_str(), hv);
                    }
                }
                builder.body(axum::body::Body::empty()).unwrap_or_default()
            }

            S3Response::GetObjectAttributes(r) => {
                let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><GetObjectAttributesResponse>"#);
                if let Some(obj) = &r.object {
                    if let Some(etag) = &obj.etag {
                        xml.push_str(&format!("<ETag>{}</ETag>", xml_escape(etag)));
                    }
                    xml.push_str(&format!("<StorageClass>{}</StorageClass>", xml_escape(obj.storage_class.as_deref().unwrap_or("STANDARD"))));
                    xml.push_str(&format!("<ObjectSize>{}</ObjectSize>", obj.size));
                }
                xml.push_str("</GetObjectAttributesResponse>");
                xml_response(&xml)
            }

            S3Response::DeleteMultipleObjects(r) => {
                let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><DeleteResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);
                for key in &r.deleted {
                    xml.push_str(&format!("<Deleted><Key>{}</Key></Deleted>", xml_escape(key)));
                }
                for e in &r.errors {
                    xml.push_str(&format!(
                        "<Error><Key>{}</Key><Code>{}</Code><Message>{}</Message></Error>",
                        xml_escape(e.resource.as_deref().unwrap_or("")),
                        xml_escape(&e.code),
                        xml_escape(&e.message),
                    ));
                }
                xml.push_str("</DeleteResult>");
                xml_response(&xml)
            }

            S3Response::PostPolicy(r) => {
                let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><PostResponse>"#);
                if let Some(loc) = &r.location { xml.push_str(&format!("<Location>{}</Location>", xml_escape(loc))); }
                if let Some(b) = &r.bucket { xml.push_str(&format!("<Bucket>{}</Bucket>", xml_escape(b))); }
                if let Some(k) = &r.key { xml.push_str(&format!("<Key>{}</Key>", xml_escape(k))); }
                if let Some(e) = &r.etag { xml.push_str(&format!("<ETag>{}</ETag>", xml_escape(e))); }
                xml.push_str("</PostResponse>");
                xml_response(&xml)
            }

            S3Response::PutObjectExtract(r) => {
                xml_response(&format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?><PutObjectExtractResult><ExtractedCount>{}</ExtractedCount></PutObjectExtractResult>"#,
                    r.extracted_count
                ))
            }

            S3Response::AppendObjectRejected(r) => {
                s3_error_xml(StatusCode::NOT_IMPLEMENTED, &r.error.code, &r.error.message)
            }

            S3Response::GetBucketPolicyStatus(r) => {
                let is_public = r.is_public.unwrap_or(false);
                xml_response(&format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?><PolicyStatus xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><IsPublic>{}</IsPublic></PolicyStatus>"#,
                    is_public
                ))
            }

            S3Response::GetBucketReplicationMetricsV2(r) => {
                let body = r.json.unwrap_or_default();
                (StatusCode::OK, [("content-type", "application/json")], body).into_response()
            }
            S3Response::GetBucketReplicationMetrics(r) => {
                let body = r.json.unwrap_or_default();
                (StatusCode::OK, [("content-type", "application/json")], body).into_response()
            }

            S3Response::ValidateBucketReplicationCreds(r) => {
                let body = format!(r#"{{"valid":{}}}"#, r.valid);
                (StatusCode::OK, [("content-type", "application/json")], body).into_response()
            }

            S3Response::ListenBucketNotification(r) => {
                let body = serde_json::to_string(&r.records).unwrap_or_default();
                (StatusCode::OK, [("content-type", "application/json")], body).into_response()
            }

            S3Response::RootListenNotification(r) => {
                let body = serde_json::to_string(&r.records).unwrap_or_default();
                (StatusCode::OK, [("content-type", "application/json")], body).into_response()
            }

            S3Response::RejectedApi(r) => {
                s3_error_xml(
                    StatusCode::from_u16(r.meta.status_code).unwrap_or(StatusCode::NOT_IMPLEMENTED),
                    &r.error.code,
                    &r.error.message,
                )
            }
        }
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}

fn xml_response(body: &str) -> Response {
    (StatusCode::OK, [("content-type", "application/xml")], body.to_string()).into_response()
}

fn xml_passthrough(xml: &Option<String>, default_root: &str) -> Response {
    let body = match xml {
        Some(x) => x.clone(),
        None => format!(r#"<?xml version="1.0" encoding="UTF-8"?><{} xmlns="http://s3.amazonaws.com/doc/2006-03-01/"></{}>"#, default_root, default_root),
    };
    (StatusCode::OK, [("content-type", "application/xml")], body).into_response()
}

fn raw_response(status: u16, meta: &ResponseMeta, body: Vec<u8>) -> Response {
    let sc = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
    let mut resp = (sc, body).into_response();
    if let Some(etag) = &meta.etag {
        if let Ok(v) = etag.parse() {
            resp.headers_mut().insert("etag", v);
        }
    }
    resp
}

fn s3_error_xml(status: StatusCode, code: &str, message: &str) -> Response {
    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Error><Code>{}</Code><Message>{}</Message></Error>"#,
        xml_escape(code), xml_escape(message)
    );
    (status, [("content-type", "application/xml")], body).into_response()
}

fn list_buckets_xml(buckets: &[BucketInfo]) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><Buckets>"#);
    for b in buckets {
        xml.push_str("<Bucket>");
        xml.push_str(&format!("<Name>{}</Name>", xml_escape(&b.name)));
        if let Some(cd) = &b.creation_date {
            xml.push_str(&format!("<CreationDate>{}</CreationDate>", xml_escape(cd)));
        }
        xml.push_str("</Bucket>");
    }
    xml.push_str("</Buckets></ListAllMyBucketsResult>");
    xml
}

fn list_objects_xml(root: &str, objects: &[ObjectInfo]) -> String {
    let mut xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?><{} xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#, root);
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
