use serde::Deserialize;

use crate::types::s3::policy::S3Action;
use crate::types::traits::s3_policyengine::{PolicyEvalContext, PolicyEffect};

/// AWS S3 风格的策略文档
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PolicyDocument {
    #[serde(default)]
    pub version: Option<String>,
    pub statement: Vec<Statement>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Statement {
    #[serde(default)]
    pub sid: Option<String>,
    pub effect: StatementEffect,
    pub principal: Option<Principal>,
    pub action: StringOrVec,
    #[serde(default)]
    pub resource: Option<StringOrVec>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum StatementEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Principal {
    Wildcard(String),
    Map(std::collections::HashMap<String, StringOrVec>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum StringOrVec {
    Single(String),
    Multiple(Vec<String>),
}

impl StringOrVec {
    pub fn as_slice(&self) -> Vec<&str> {
        match self {
            Self::Single(s) => vec![s.as_str()],
            Self::Multiple(v) => v.iter().map(|s| s.as_str()).collect(),
        }
    }
}

impl PolicyDocument {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 评估策略文档，返回 None 表示无匹配语句
    pub fn evaluate(&self, ctx: &PolicyEvalContext) -> Option<PolicyEffect> {
        let mut has_allow = false;

        for stmt in &self.statement {
            if !stmt.matches_action(ctx.action) {
                continue;
            }
            if !stmt.matches_principal(&ctx.identity) {
                continue;
            }
            if !stmt.matches_resource(ctx.bucket.as_deref(), ctx.key.as_deref()) {
                continue;
            }
            match stmt.effect {
                StatementEffect::Deny => return Some(PolicyEffect::Deny),
                StatementEffect::Allow => has_allow = true,
            }
        }

        if has_allow { Some(PolicyEffect::Allow) } else { None }
    }
}

impl Statement {
    fn matches_action(&self, action: S3Action) -> bool {
        let action_str = action.as_str();
        self.action.as_slice().iter().any(|pat| {
            *pat == "*" || *pat == "s3:*" || *pat == action_str
        })
    }

    fn matches_principal(&self, identity: &str) -> bool {
        match &self.principal {
            None => true,
            Some(Principal::Wildcard(s)) => s == "*",
            Some(Principal::Map(m)) => {
                m.values().any(|v| {
                    v.as_slice().iter().any(|p| *p == "*" || *p == identity)
                })
            }
        }
    }

    fn matches_resource(&self, bucket: Option<&str>, key: Option<&str>) -> bool {
        let resource = match &self.resource {
            None => return true,
            Some(r) => r,
        };
        let arn = match (bucket, key) {
            (Some(b), Some(k)) => format!("arn:aws:s3:::{b}/{k}"),
            (Some(b), None) => format!("arn:aws:s3:::{b}"),
            _ => return true,
        };
        resource.as_slice().iter().any(|pat| match_resource_pattern(pat, &arn))
    }
}

/// 简单的 ARN 通配符匹配（支持 * 和 ?）
fn match_resource_pattern(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let mut p = pattern.chars().peekable();
    let mut v = value.chars().peekable();
    match_wildcard(&mut p, &mut v)
}

fn match_wildcard(
    p: &mut std::iter::Peekable<std::str::Chars>,
    v: &mut std::iter::Peekable<std::str::Chars>,
) -> bool {
    while let Some(&pc) = p.peek() {
        match pc {
            '*' => {
                p.next();
                if p.peek().is_none() {
                    return true;
                }
                // 尝试匹配 * 后的剩余部分
                loop {
                    let mut p2 = p.clone();
                    let mut v2 = v.clone();
                    if match_wildcard(&mut p2, &mut v2) {
                        return true;
                    }
                    if v.next().is_none() {
                        return false;
                    }
                }
            }
            '?' => {
                p.next();
                if v.next().is_none() {
                    return false;
                }
            }
            _ => {
                p.next();
                match v.next() {
                    Some(vc) if vc == pc => {}
                    _ => return false,
                }
            }
        }
    }
    v.peek().is_none()
}
