use std::fmt::{Display, Formatter};

use crate::types::traits::BoxError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FS3Error(pub String);

impl Display for FS3Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FS3Error {}

impl From<&str> for FS3Error {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for FS3Error {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<std::io::Error> for FS3Error {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<serde_json::Error> for FS3Error {
    fn from(value: serde_json::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<quick_xml::DeError> for FS3Error {
    fn from(value: quick_xml::DeError) -> Self {
        Self(value.to_string())
    }
}

impl From<chrono::ParseError> for FS3Error {
    fn from(value: chrono::ParseError) -> Self {
        Self(value.to_string())
    }
}

impl From<BoxError> for FS3Error {
    fn from(value: BoxError) -> Self {
        Self(value.to_string())
    }
}
