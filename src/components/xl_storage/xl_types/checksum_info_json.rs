use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumInfoJson {
    pub name: String,
    pub algorithm: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}
