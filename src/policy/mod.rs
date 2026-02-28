use crate::config::PolicyGroup;

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    groups: Vec<PolicyGroup>,
}

impl PolicyEngine {
    pub fn new(groups: &[PolicyGroup]) -> Self {
        Self {
            groups: groups.to_vec(),
        }
    }

    pub fn allowed(&self, user: &str, bucket: &str, key: &str) -> bool {
        let mut allowed = false;
        for g in &self.groups {
            if !g.enabled {
                continue;
            }
            if !match_user(&g.users, user) {
                continue;
            }
            for rule in &g.rules {
                if !rule.bucket.is_empty() && rule.bucket != bucket {
                    continue;
                }
                if !rule.prefix.is_empty() && !key.starts_with(&rule.prefix) {
                    continue;
                }
                if !match_user(&rule.users, user) {
                    continue;
                }
                allowed = rule.allow;
            }
        }
        allowed
    }
}

fn match_user(users: &[String], user: &str) -> bool {
    if users.is_empty() {
        return true;
    }
    users.iter().any(|u| u == user)
}
