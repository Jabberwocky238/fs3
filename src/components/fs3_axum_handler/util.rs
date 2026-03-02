use std::collections::HashMap;

use axum::http::HeaderMap;

use crate::types::s3::request::{EventFilter, ListQuery, MultipartSelector};

pub fn has(q: &HashMap<String, String>, key: &str) -> bool { q.contains_key(key) }
pub fn get(q: &HashMap<String, String>, key: &str) -> Option<String> { q.get(key).cloned() }

pub fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers.get(key).and_then(|v| v.to_str().ok()).map(ToString::to_string)
}

pub fn header_eq(headers: &HeaderMap, key: &str, value: &str) -> bool {
    header(headers, key).map(|v| v.eq_ignore_ascii_case(value)).unwrap_or(false)
}

pub fn body_string(body: &[u8]) -> Option<String> {
    if body.is_empty() { None } else { Some(String::from_utf8_lossy(body).to_string()) }
}

pub fn event_filter(q: &HashMap<String, String>) -> EventFilter {
    EventFilter {
        events: q.get("events")
            .map(|v| v.split(',').filter(|s| !s.trim().is_empty()).map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        prefix: q.get("prefix").cloned(),
        suffix: q.get("suffix").cloned(),
    }
}

pub fn list_query(q: &HashMap<String, String>) -> ListQuery {
    ListQuery {
        prefix: q.get("prefix").cloned(),
        delimiter: q.get("delimiter").cloned(),
        max_keys: q.get("max-keys").and_then(|v| v.parse::<u32>().ok()),
        continuation_token: q.get("continuation-token").cloned(),
        start_after: q.get("start-after").cloned(),
        marker: q.get("marker").cloned(),
        version_id_marker: q.get("version-id-marker").cloned(),
        key_marker: q.get("key-marker").cloned(),
    }
}

pub fn multipart_selector(q: &HashMap<String, String>) -> MultipartSelector {
    MultipartSelector {
        upload_id: get(q, "uploadId").unwrap_or_default(),
        part_number: q.get("partNumber").and_then(|v| v.parse::<u32>().ok()),
    }
}
