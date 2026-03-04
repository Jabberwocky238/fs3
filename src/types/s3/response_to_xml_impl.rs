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
