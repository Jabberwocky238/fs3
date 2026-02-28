use axum::http::HeaderMap;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthContext {
    pub access_key: String,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub enabled: bool,
    pub access_key: String,
    pub secret_key: String,
}

pub fn extract_access_key(headers: &HeaderMap, query_credential: Option<&str>) -> Option<String> {
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok())
        && let Some(pos) = auth.find("Credential=")
    {
        let rem = &auth[pos + "Credential=".len()..];
        let key = rem
            .split('/')
            .next()
            .unwrap_or("")
            .split(',')
            .next()
            .unwrap_or("")
            .trim();
        if !key.is_empty() {
            return Some(key.to_string());
        }
    }

    if let Some(cred) = query_credential {
        let key = cred.split('/').next().unwrap_or("").trim();
        if !key.is_empty() {
            return Some(key.to_string());
        }
    }

    None
}

pub fn check_access_key(
    users: &[AuthUser],
    headers: &HeaderMap,
    query_credential: Option<&str>,
) -> Result<AuthContext, &'static str> {
    let Some(input_key) = extract_access_key(headers, query_credential) else {
        warn!("auth failed: missing access key");
        return Err("missing access key");
    };

    let ok = users
        .iter()
        .any(|u| u.enabled && !u.access_key.is_empty() && u.access_key == input_key);

    if ok {
        debug!(access_key = %input_key, "auth ok");
        Ok(AuthContext {
            access_key: input_key,
        })
    } else {
        warn!(access_key = %input_key, "auth failed: invalid access key");
        Err("invalid access key")
    }
}
