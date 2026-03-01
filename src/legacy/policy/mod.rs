mod types;
pub use types::*;

/// Context for a policy evaluation request.
pub struct PolicyRequest<'a> {
    pub user: &'a str,
    pub action: &'a str,
    pub bucket: &'a str,
    pub key: &'a str,
    pub visibility: Option<Visibility>,
}

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    policys: Vec<Policy>,
}

impl PolicyEngine {
    pub fn new(policys: Vec<Policy>) -> Self {
        Self {
            policys,
        }
    }

    /// Evaluate whether the request is allowed.
    ///
    /// Default deny. Explicit Deny always overrides Allow.
    pub fn evaluate(&self, req: &PolicyRequest) -> bool {
        let mut has_allow = false;
        let mut has_deny = false;

        for policy in &self.policys {
            match policy {
                Policy::Privilege(p) => {
                    if !match_user(&p.users, req.user) {
                        continue;
                    }
                    if !match_action(&p.actions, req.action) {
                            continue;
                        }
                        match p.effect {
                            Effect::Allow => has_allow = true,
                            Effect::Deny => has_deny = true,
                        }
                    }
                    Policy::Resource(r) => {
                        if !match_user(&r.users, req.user) {
                            continue;
                        }
                        if !match_resource(r, req.bucket, req.key, req.visibility) {
                            continue;
                        }
                        match r.effect {
                            Effect::Allow => has_allow = true,
                            Effect::Deny => has_deny = true,
                        }
                    }
                }
            }
        }

        has_allow && !has_deny
    }

    /// Backward-compatible helper used by the open gateway.
    pub fn allowed(&self, user: &str, bucket: &str, key: &str) -> bool {
        self.evaluate(&PolicyRequest {
            user,
            action: "object.get",
            bucket,
            key,
            visibility: None,
        })
    }
}

fn match_user(users: &[String], user: &str) -> bool {
    if users.is_empty() {
        return true;
    }
    users.iter().any(|u| {
        if u == "*" || u == user {
            return true;
        }
        // "user:<userid>" syntax
        if let Some(id) = u.strip_prefix("user:") {
            return id == user;
        }
        false
    })
}

fn match_action(actions: &[String], action: &str) -> bool {
    actions.iter().any(|a| {
        if a == "*" || a == action {
            return true;
        }
        // Namespace wildcard: "bucket.*" matches "bucket.list", etc.
        if let Some(ns) = a.strip_suffix(".*") {
            if let Some(act_ns) = action.split('.').next() {
                return ns == act_ns;
            }
        }
        false
    })
}

fn match_resource(res: &ResourcePolicy, bucket: &str, key: &str, vis: Option<Visibility>) -> bool {
    if !res.bucket.is_empty() && res.bucket != bucket {
        return false;
    }
    if !res.prefix.is_empty() && !key.starts_with(&res.prefix) {
        return false;
    }
    if let Some(required) = res.visibility {
        match vis {
            Some(actual) if actual == required => {}
            None => {}
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_deny() {
        let engine = PolicyEngine::new(vec![]);
        assert!(!engine.allowed("alice", "photos", "cat.jpg"));
    }

    #[test]
    fn privilege_allow() {
        let engine = PolicyEngine::new(vec![
            Policy::Privilege(PrivilegePolicy {
                actions: vec!["object.get".into()],
                effect: Effect::Allow,
                users: vec!["alice".into()],
            }),
        ]);
        assert!(engine.allowed("alice", "photos", "cat.jpg"));
        assert!(!engine.allowed("bob", "photos", "cat.jpg"));
    }

    #[test]
    fn privilege_deny_overrides() {
        let engine = PolicyEngine::new(vec![
            Policy::Privilege(PrivilegePolicy {
                actions: vec!["object.*".into()],
                effect: Effect::Allow,
                users: vec![],
            }),
            Policy::Privilege(PrivilegePolicy {
                actions: vec!["object.get".into()],
                effect: Effect::Deny,
                users: vec!["bob".into()],
            }),
        ]);
        assert!(engine.allowed("alice", "any", "file.txt"));
        assert!(!engine.allowed("bob", "any", "file.txt"));
    }

    #[test]
    fn namespace_wildcard() {
        let engine = PolicyEngine::new(&[group(vec![
            Policy::Privilege(PrivilegePolicy {
                actions: vec!["bucket.*".into()],
                effect: Effect::Allow,
                users: vec![],
            }),
        ])]);
        let req = PolicyRequest {
            user: "alice", action: "bucket.list",
            bucket: "", key: "", visibility: None,
        };
        assert!(engine.evaluate(&req));

        let req2 = PolicyRequest {
            user: "alice", action: "object.get",
            bucket: "", key: "", visibility: None,
        };
        assert!(!engine.evaluate(&req2));
    }

    #[test]
    fn resource_bucket_deny() {
        let engine = PolicyEngine::new(&[group(vec![
            Policy::Privilege(PrivilegePolicy {
                actions: vec!["*".into()],
                effect: Effect::Allow,
                users: vec![],
            }),
            Policy::Resource(ResourcePolicy {
                effect: Effect::Deny,
                users: vec!["guest".into()],
                bucket: "internal".into(),
                prefix: String::new(),
                visibility: None,
            }),
        ])]);
        assert!(engine.allowed("admin", "internal", "doc.pdf"));
        assert!(!engine.allowed("guest", "internal", "doc.pdf"));
        assert!(engine.allowed("guest", "public", "doc.pdf"));
    }

    #[test]
    fn resource_visibility_filter() {
        let engine = PolicyEngine::new(&[group(vec![
            Policy::Privilege(PrivilegePolicy {
                actions: vec!["object.get".into()],
                effect: Effect::Allow,
                users: vec![],
            }),
            Policy::Resource(ResourcePolicy {
                effect: Effect::Deny,
                users: vec![],
                bucket: String::new(),
                prefix: String::new(),
                visibility: Some(Visibility::Private),
            }),
        ])]);

        let pub_req = PolicyRequest {
            user: "alice", action: "object.get",
            bucket: "b", key: "k",
            visibility: Some(Visibility::Public),
        };
        assert!(engine.evaluate(&pub_req));

        let priv_req = PolicyRequest {
            user: "alice", action: "object.get",
            bucket: "b", key: "k",
            visibility: Some(Visibility::Private),
        };
        assert!(!engine.evaluate(&priv_req));
    }

    #[test]
    fn resource_prefix_match() {
        let engine = PolicyEngine::new(&[group(vec![
            Policy::Privilege(PrivilegePolicy {
                actions: vec!["object.get".into()],
                effect: Effect::Allow,
                users: vec![],
            }),
            Policy::Resource(ResourcePolicy {
                effect: Effect::Deny,
                users: vec![],
                bucket: "data".into(),
                prefix: "secret/".into(),
                visibility: None,
            }),
        ])]);
        assert!(engine.allowed("alice", "data", "reports/q1.csv"));
        assert!(!engine.allowed("alice", "data", "secret/key.pem"));
    }

    #[test]
    fn serde_roundtrip() {
        let p = Policy::Privilege(PrivilegePolicy {
            actions: vec!["bucket.list".into(), "bucket.create".into()],
            effect: Effect::Allow,
            users: vec!["alice".into()],
        });
        let json = serde_json::to_string(&p).unwrap();
        let p2: Policy = serde_json::from_str(&json).unwrap();
        assert_eq!(p, p2);

        let r = Policy::Resource(ResourcePolicy {
            effect: Effect::Deny,
            users: vec![],
            bucket: "internal".into(),
            prefix: String::new(),
            visibility: Some(Visibility::Private),
        });
        let json2 = serde_json::to_string(&r).unwrap();
        let r2: Policy = serde_json::from_str(&json2).unwrap();
        assert_eq!(r, r2);
    }
}
