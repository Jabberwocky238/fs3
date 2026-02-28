use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct PolicyGroup {
    pub name: String,
    pub users: Vec<String>,
    pub rules: Vec<PolicyRule>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct PolicyRule {
    pub bucket: String,
    pub prefix: String,
    pub allow: bool,
    pub users: Vec<String>,
}

