use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::types::s3::core::{
    BucketEncryption, BucketObjectLockConfig, BucketReplication, BucketVersioning,
    BucketWebsite, CompleteMultipartInput, CorsConfiguration, ObjectLegalHold, ObjectLockMode,
    ObjectRetention, UploadedPart,
};

use crate::types::traits::s3_handler::S3HandlerBridgeError;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccessControlPolicyInput {
    pub owner_id: Option<String>,
    pub owner_display_name: Option<String>,
    pub grants: Vec<AccessControlGrantInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccessControlGrantInput {
    pub permission: Option<String>,
    pub grantee_uri: Option<String>,
    pub grantee_id: Option<String>,
    pub grantee_display_name: Option<String>,
    pub grantee_email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeleteObjectsInputXml {
    pub quiet: bool,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LifecycleRuleInput {
    pub id: Option<String>,
    pub status: Option<String>,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NotificationConfigInput {
    pub target_arn: String,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SelectObjectContentInput {
    pub expression: Option<String>,
    pub expression_type: Option<String>,
    pub input_serialization: Option<String>,
    pub output_serialization: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RestoreObjectInput {
    pub days: Option<u32>,
    pub tier: Option<String>,
}

pub fn parse_access_control_policy(xml: &str) -> Result<AccessControlPolicyInput, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct AccessControlPolicyXml {
        #[serde(rename = "Owner")]
        owner: Option<OwnerXml>,
        #[serde(rename = "AccessControlList")]
        access_control_list: Option<AccessControlListXml>,
    }

    #[derive(Deserialize)]
    struct OwnerXml {
        #[serde(rename = "ID")]
        id: Option<String>,
        #[serde(rename = "DisplayName")]
        display_name: Option<String>,
    }

    #[derive(Deserialize)]
    struct AccessControlListXml {
        #[serde(rename = "Grant", default)]
        grants: Vec<GrantXml>,
    }

    #[derive(Deserialize)]
    struct GrantXml {
        #[serde(rename = "Permission")]
        permission: Option<String>,
        #[serde(rename = "Grantee")]
        grantee: Option<GranteeXml>,
    }

    #[derive(Deserialize)]
    struct GranteeXml {
        #[serde(rename = "URI")]
        uri: Option<String>,
        #[serde(rename = "ID")]
        id: Option<String>,
        #[serde(rename = "DisplayName")]
        display_name: Option<String>,
        #[serde(rename = "EmailAddress")]
        email: Option<String>,
    }

    let parsed: AccessControlPolicyXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(AccessControlPolicyInput {
        owner_id: parsed.owner.as_ref().and_then(|owner| owner.id.clone()),
        owner_display_name: parsed.owner.and_then(|owner| owner.display_name),
        grants: parsed
            .access_control_list
            .map(|acl| {
                acl.grants
                    .into_iter()
                    .map(|grant| AccessControlGrantInput {
                        permission: grant.permission,
                        grantee_uri: grant.grantee.as_ref().and_then(|grantee| grantee.uri.clone()),
                        grantee_id: grant.grantee.as_ref().and_then(|grantee| grantee.id.clone()),
                        grantee_display_name: grant
                            .grantee
                            .as_ref()
                            .and_then(|grantee| grantee.display_name.clone()),
                        grantee_email: grant.grantee.and_then(|grantee| grantee.email),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    })
}

pub fn parse_complete_multipart_upload(
    xml: &str,
) -> Result<CompleteMultipartInput, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct CompleteMultipartUploadXml {
        #[serde(rename = "Part", default)]
        parts: Vec<CompletedPartXml>,
    }

    #[derive(Deserialize)]
    struct CompletedPartXml {
        #[serde(rename = "PartNumber")]
        part_number: u32,
        #[serde(rename = "ETag")]
        etag: Option<String>,
    }

    let parsed: CompleteMultipartUploadXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(CompleteMultipartInput {
        parts: parsed
            .parts
            .into_iter()
            .map(|part| UploadedPart {
                part_number: part.part_number,
                etag: part.etag.unwrap_or_default(),
                size: 0,
            })
            .collect(),
    })
}

pub fn parse_delete_objects(xml: &str) -> Result<DeleteObjectsInputXml, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct DeleteXml {
        #[serde(rename = "Quiet")]
        quiet: Option<bool>,
        #[serde(rename = "Object", default)]
        objects: Vec<DeleteObjectXml>,
    }

    #[derive(Deserialize)]
    struct DeleteObjectXml {
        #[serde(rename = "Key")]
        key: String,
    }

    let parsed: DeleteXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(DeleteObjectsInputXml {
        quiet: parsed.quiet.unwrap_or(false),
        keys: parsed.objects.into_iter().map(|object| object.key).collect(),
    })
}

pub fn parse_tagging(xml: &str) -> Result<HashMap<String, String>, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct TaggingXml {
        #[serde(rename = "TagSet")]
        tag_set: TagSetXml,
    }

    #[derive(Deserialize)]
    struct TagSetXml {
        #[serde(rename = "Tag", default)]
        tags: Vec<TagXml>,
    }

    #[derive(Deserialize)]
    struct TagXml {
        #[serde(rename = "Key")]
        key: String,
        #[serde(rename = "Value")]
        value: String,
    }

    let parsed: TaggingXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(parsed
        .tag_set
        .tags
        .into_iter()
        .map(|tag| (tag.key, tag.value))
        .collect())
}

pub fn parse_retention(xml: &str) -> Result<ObjectRetention, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct RetentionXml {
        #[serde(rename = "Mode")]
        mode: String,
        #[serde(rename = "RetainUntilDate")]
        retain_until_date: String,
    }

    let parsed: RetentionXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    let mode = match parsed.mode.as_str() {
        "GOVERNANCE" => ObjectLockMode::Governance,
        "COMPLIANCE" => ObjectLockMode::Compliance,
        other => {
            return Err(S3HandlerBridgeError::InvalidRequest(format!(
                "invalid retention mode: {other}"
            )));
        }
    };
    let retain_until = DateTime::parse_from_rfc3339(&parsed.retain_until_date)
        .map_err(|e| S3HandlerBridgeError::InvalidRequest(format!("invalid retain-until date: {e}")))?
        .with_timezone(&Utc);
    Ok(ObjectRetention { mode, retain_until })
}

pub fn parse_legal_hold(xml: &str) -> Result<ObjectLegalHold, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct LegalHoldXml {
        #[serde(rename = "Status")]
        status: String,
    }

    let parsed: LegalHoldXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(ObjectLegalHold {
        enabled: matches!(parsed.status.as_str(), "ON" | "On" | "on"),
    })
}

pub fn parse_lifecycle(xml: &str) -> Result<Vec<LifecycleRuleInput>, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct LifecycleConfigurationXml {
        #[serde(rename = "Rule", default)]
        rules: Vec<LifecycleRuleXml>,
    }

    #[derive(Deserialize)]
    struct LifecycleRuleXml {
        #[serde(rename = "ID")]
        id: Option<String>,
        #[serde(rename = "Status")]
        status: Option<String>,
        #[serde(rename = "Filter")]
        filter: Option<LifecycleFilterXml>,
        #[serde(rename = "Prefix")]
        prefix: Option<String>,
    }

    #[derive(Deserialize)]
    struct LifecycleFilterXml {
        #[serde(rename = "Prefix")]
        prefix: Option<String>,
    }

    let parsed: LifecycleConfigurationXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(parsed
        .rules
        .into_iter()
        .map(|rule| LifecycleRuleInput {
            id: rule.id,
            status: rule.status,
            prefix: rule.prefix.or_else(|| rule.filter.and_then(|filter| filter.prefix)),
        })
        .collect())
}

pub fn parse_encryption(xml: &str) -> Result<BucketEncryption, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct EncryptionXml {
        #[serde(rename = "Rule", default)]
        rules: Vec<EncryptionRuleXml>,
    }

    #[derive(Deserialize)]
    struct EncryptionRuleXml {
        #[serde(rename = "ApplyServerSideEncryptionByDefault")]
        default: EncryptionDefaultXml,
    }

    #[derive(Deserialize)]
    struct EncryptionDefaultXml {
        #[serde(rename = "SSEAlgorithm")]
        algorithm: String,
        #[serde(rename = "KMSMasterKeyID")]
        key_id: Option<String>,
    }

    let parsed: EncryptionXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    let default = parsed
        .rules
        .into_iter()
        .next()
        .ok_or_else(|| S3HandlerBridgeError::InvalidRequest("missing encryption rule".to_string()))?
        .default;
    Ok(BucketEncryption {
        algorithm: default.algorithm,
        key_id: default.key_id,
    })
}

pub fn parse_object_lock(xml: &str) -> Result<BucketObjectLockConfig, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct ObjectLockConfigurationXml {
        #[serde(rename = "ObjectLockEnabled")]
        object_lock_enabled: Option<String>,
        #[serde(rename = "Rule")]
        rule: Option<ObjectLockRuleXml>,
    }

    #[derive(Deserialize)]
    struct ObjectLockRuleXml {
        #[serde(rename = "DefaultRetention")]
        default_retention: Option<DefaultRetentionXml>,
    }

    #[derive(Deserialize)]
    struct DefaultRetentionXml {
        #[serde(rename = "Mode")]
        mode: Option<String>,
        #[serde(rename = "Days")]
        days: Option<u32>,
        #[serde(rename = "Years")]
        years: Option<u32>,
    }

    let parsed: ObjectLockConfigurationXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    let retention = parsed.rule.and_then(|rule| rule.default_retention);
    Ok(BucketObjectLockConfig {
        enabled: matches!(
            parsed.object_lock_enabled.as_deref(),
            Some("Enabled") | Some("enabled")
        ),
        mode: retention.as_ref().and_then(|item| item.mode.clone()),
        days: retention.as_ref().and_then(|item| item.days),
        years: retention.and_then(|item| item.years),
    })
}

pub fn parse_versioning(xml: &str) -> Result<BucketVersioning, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct VersioningConfigurationXml {
        #[serde(rename = "Status")]
        status: Option<String>,
        #[serde(rename = "MfaDelete")]
        mfa_delete: Option<String>,
    }

    let parsed: VersioningConfigurationXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(BucketVersioning {
        status: parsed
            .status
            .ok_or_else(|| S3HandlerBridgeError::InvalidVersioningStatus("missing status".to_string()))?,
        mfa_delete: parsed.mfa_delete,
    })
}

pub fn parse_replication(xml: &str) -> Result<BucketReplication, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct ReplicationConfigurationXml {
        #[serde(rename = "Role")]
        role: Option<String>,
        #[serde(rename = "Rule", default)]
        rules: Vec<ReplicationRuleXml>,
    }

    #[derive(Deserialize)]
    struct ReplicationRuleXml {
        #[serde(rename = "ID")]
        id: Option<String>,
        #[serde(rename = "Status")]
        status: Option<String>,
        #[serde(rename = "Priority")]
        priority: Option<u32>,
    }

    let parsed: ReplicationConfigurationXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(BucketReplication {
        role: parsed.role.unwrap_or_default(),
        rules: parsed
            .rules
            .into_iter()
            .map(|rule| {
                let mut parts = Vec::new();
                if let Some(id) = rule.id {
                    parts.push(format!("id={id}"));
                }
                if let Some(status) = rule.status {
                    parts.push(format!("status={status}"));
                }
                if let Some(priority) = rule.priority {
                    parts.push(format!("priority={priority}"));
                }
                parts.join(",")
            })
            .collect(),
    })
}

pub fn parse_notification(xml: &str) -> Result<Vec<NotificationConfigInput>, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct NotificationConfigurationXml {
        #[serde(rename = "TopicConfiguration", default)]
        topics: Vec<NotificationTargetXml>,
        #[serde(rename = "QueueConfiguration", default)]
        queues: Vec<NotificationTargetXml>,
        #[serde(rename = "CloudFunctionConfiguration", default)]
        functions: Vec<NotificationTargetXml>,
    }

    #[derive(Deserialize)]
    struct NotificationTargetXml {
        #[serde(rename = "Topic")]
        topic: Option<String>,
        #[serde(rename = "Queue")]
        queue: Option<String>,
        #[serde(rename = "CloudFunction")]
        cloud_function: Option<String>,
        #[serde(rename = "Event", default)]
        events: Vec<String>,
    }

    let parsed: NotificationConfigurationXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(parsed
        .topics
        .into_iter()
        .chain(parsed.queues)
        .chain(parsed.functions)
        .map(|item| NotificationConfigInput {
            target_arn: item.topic.or(item.queue).or(item.cloud_function).unwrap_or_default(),
            events: item.events,
        })
        .collect())
}

pub fn parse_website(xml: &str) -> Result<BucketWebsite, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct WebsiteConfigurationXml {
        #[serde(rename = "IndexDocument")]
        index_document: Option<IndexDocumentXml>,
        #[serde(rename = "ErrorDocument")]
        error_document: Option<ErrorDocumentXml>,
    }

    #[derive(Deserialize)]
    struct IndexDocumentXml {
        #[serde(rename = "Suffix")]
        suffix: String,
    }

    #[derive(Deserialize)]
    struct ErrorDocumentXml {
        #[serde(rename = "Key")]
        key: String,
    }

    let parsed: WebsiteConfigurationXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(BucketWebsite {
        index_document: parsed
            .index_document
            .ok_or_else(|| S3HandlerBridgeError::InvalidRequest("missing index document".to_string()))?
            .suffix,
        error_document: parsed.error_document.map(|item| item.key),
    })
}

pub fn parse_cors(xml: &str) -> Result<CorsConfiguration, S3HandlerBridgeError> {
    quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))
}

pub fn parse_select_object_content(
    xml: &str,
) -> Result<SelectObjectContentInput, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct SelectRequestXml {
        #[serde(rename = "Expression")]
        expression: Option<String>,
        #[serde(rename = "ExpressionType")]
        expression_type: Option<String>,
        #[serde(rename = "InputSerialization")]
        input_serialization: Option<AnyXmlNode>,
        #[serde(rename = "OutputSerialization")]
        output_serialization: Option<AnyXmlNode>,
    }

    #[derive(Deserialize)]
    struct AnyXmlNode {
        #[serde(rename = "$value", default)]
        values: Vec<String>,
    }

    let parsed: SelectRequestXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(SelectObjectContentInput {
        expression: parsed.expression,
        expression_type: parsed.expression_type,
        input_serialization: parsed.input_serialization.map(|node| node.values.join("")),
        output_serialization: parsed.output_serialization.map(|node| node.values.join("")),
    })
}

pub fn parse_restore_object(xml: &str) -> Result<RestoreObjectInput, S3HandlerBridgeError> {
    #[derive(Deserialize)]
    struct RestoreRequestXml {
        #[serde(rename = "Days")]
        days: Option<u32>,
        #[serde(rename = "GlacierJobParameters")]
        glacier_job_parameters: Option<GlacierJobParametersXml>,
    }

    #[derive(Deserialize)]
    struct GlacierJobParametersXml {
        #[serde(rename = "Tier")]
        tier: Option<String>,
    }

    let parsed: RestoreRequestXml =
        quick_xml::de::from_str(xml).map_err(|e| S3HandlerBridgeError::XmlParse(e.to_string()))?;
    Ok(RestoreObjectInput {
        days: parsed.days,
        tier: parsed.glacier_job_parameters.and_then(|params| params.tier),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tagging() {
        let xml = r#"<Tagging><TagSet><Tag><Key>a</Key><Value>b</Value></Tag></TagSet></Tagging>"#;
        let tags = parse_tagging(xml).unwrap();
        assert_eq!(tags.get("a"), Some(&"b".to_string()));
    }

    #[test]
    fn parses_complete_multipart_upload() {
        let xml = r#"<CompleteMultipartUpload><Part><PartNumber>1</PartNumber><ETag>abc</ETag></Part></CompleteMultipartUpload>"#;
        let input = parse_complete_multipart_upload(xml).unwrap();
        assert_eq!(input.parts.len(), 1);
        assert_eq!(input.parts[0].part_number, 1);
        assert_eq!(input.parts[0].etag, "abc");
    }

    #[test]
    fn parses_retention() {
        let xml = r#"<Retention><Mode>GOVERNANCE</Mode><RetainUntilDate>2026-03-12T00:00:00Z</RetainUntilDate></Retention>"#;
        let retention = parse_retention(xml).unwrap();
        assert!(matches!(retention.mode, ObjectLockMode::Governance));
    }
}
